//! Declarative JSX syntax for CreamUI's Rust widget API.
//!
//! `jsx!` is deliberately a thin syntax layer: it emits calls to
//! `creamui_widgets` and does not own state, rendering, or an ABI.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    braced, parse::Parse, parse::ParseStream, parse_macro_input, Error, Expr, FnArg, Ident, ItemFn,
    LitStr, Pat, Result, Token,
};

/// Builds a CreamUI widget using JSX-like syntax.
///
/// See the workspace README for the supported component and prop mapping.
#[proc_macro]
pub fn jsx(input: TokenStream) -> TokenStream {
    let element = parse_macro_input!(input as Element);
    match element.expand() {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

/// Turns a normal Rust function into a JSX component.
///
/// The function's named arguments become fields in a generated `NameProps`
/// struct. `<Name value={...}/>` then expands to `Name(NameProps { value })`.
#[proc_macro_attribute]
pub fn component(_attribute: TokenStream, input: TokenStream) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);
    match expand_component(function) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_component(function: ItemFn) -> Result<TokenStream2> {
    if function.sig.receiver().is_some() {
        return Err(Error::new_spanned(
            function.sig,
            "`#[component]` only supports free functions",
        ));
    }
    if function.sig.inputs.is_empty() {
        return Err(Error::new_spanned(
            function.sig,
            "a JSX component needs at least one named prop",
        ));
    }
    let function_name = &function.sig.ident;
    let props_name = format_ident!("{}Props", function_name);
    let visibility = &function.vis;
    let attributes = &function.attrs;
    let output = &function.sig.output;
    let block = &function.block;
    let mut fields = Vec::new();
    let mut bindings = Vec::new();
    for argument in &function.sig.inputs {
        let FnArg::Typed(argument) = argument else {
            unreachable!("receiver checked above");
        };
        let Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(Error::new_spanned(
                &argument.pat,
                "component props must be simple identifiers",
            ));
        };
        let name = &pattern.ident;
        let ty = &argument.ty;
        fields.push(quote!(pub #name: #ty));
        bindings.push(quote!(#name));
    }
    Ok(quote! {
        #visibility struct #props_name { #( #fields, )* }
        #( #attributes )*
        #[allow(non_snake_case)]
        #visibility fn #function_name(props: #props_name) #output {
            let #props_name { #( #bindings, )* } = props;
            #block
        }
    })
}

struct Attribute {
    name: Ident,
    value: Expr,
}

enum Child {
    Element(Element),
    Expression(Expr),
    Text(LitStr),
}

struct Element {
    tag: Ident,
    attributes: Vec<Attribute>,
    children: Vec<Child>,
}

impl Parse for Element {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        input.parse::<Token![<]>()?;
        let tag: Ident = input.parse()?;
        let mut attributes = Vec::new();

        while !input.peek(Token![>]) && !input.peek(Token![/]) {
            let name: Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let content;
            braced!(content in input);
            attributes.push(Attribute {
                name,
                value: content.parse()?,
            });
        }

        if input.peek(Token![/]) {
            input.parse::<Token![/]>()?;
            input.parse::<Token![>]>()?;
            return Ok(Element {
                tag,
                attributes,
                children: Vec::new(),
            });
        }
        input.parse::<Token![>]>()?;

        let mut children = Vec::new();
        while !is_closing_tag(input) {
            if input.peek(Token![<]) {
                children.push(Child::Element(input.parse()?));
            } else if input.peek(syn::token::Brace) {
                let content;
                braced!(content in input);
                children.push(Child::Expression(content.parse()?));
            } else if input.peek(LitStr) {
                children.push(Child::Text(input.parse()?));
            } else {
                return Err(input.error("JSX children must be an element, a Rust expression in `{...}`, or a string literal"));
            }
        }

        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let closing_tag: Ident = input.parse()?;
        if closing_tag != tag {
            return Err(Error::new_spanned(
                closing_tag,
                format!("expected closing tag `</{tag}>`"),
            ));
        }
        input.parse::<Token![>]>()?;
        Ok(Element {
            tag,
            attributes,
            children,
        })
    }
}

fn is_closing_tag(input: ParseStream<'_>) -> bool {
    if !input.peek(Token![<]) {
        return false;
    }
    let fork = input.fork();
    let _ = fork.parse::<Token![<]>();
    fork.peek(Token![/])
}

impl Element {
    fn prop(&self, name: &str) -> Result<Option<Expr>> {
        let mut result = None;
        for attribute in &self.attributes {
            if attribute.name == name {
                if result.is_some() {
                    return Err(Error::new_spanned(
                        &attribute.name,
                        format!("duplicate `{name}` prop"),
                    ));
                }
                result = Some(attribute.value.clone());
            }
        }
        Ok(result)
    }

    fn required_prop(&self, name: &str) -> Result<Expr> {
        self.prop(name)?.ok_or_else(|| {
            Error::new_spanned(
                &self.tag,
                format!("`{}` requires a `{name}` prop", self.tag),
            )
        })
    }

    fn reject_unknown_props(&self, allowed: &[&str]) -> Result<()> {
        for attribute in &self.attributes {
            if !allowed.iter().any(|allowed| attribute.name == allowed) {
                return Err(Error::new_spanned(
                    &attribute.name,
                    format!(
                        "`{}` does not support the `{}` prop",
                        self.tag, attribute.name
                    ),
                ));
            }
        }
        Ok(())
    }

    fn text_child(&self) -> Result<TokenStream2> {
        if self.children.len() != 1 {
            return Err(Error::new_spanned(
                &self.tag,
                format!("`{}` requires exactly one text child", self.tag),
            ));
        }
        match &self.children[0] {
            Child::Expression(expression) => Ok(quote!(#expression)),
            Child::Text(text) => Ok(quote!(#text)),
            Child::Element(element) => Err(Error::new_spanned(
                &element.tag,
                format!("`{}` accepts text, not child elements", self.tag),
            )),
        }
    }

    fn container_children(&self, initial: TokenStream2) -> Result<TokenStream2> {
        let mut output = initial;
        for child in &self.children {
            let child = match child {
                Child::Element(element) => {
                    let expanded = element.expand()?;
                    quote!(::creamui_jsx::IntoWidget::into_widget(#expanded))
                }
                Child::Expression(expression) => {
                    quote!(::creamui_jsx::IntoWidget::into_widget(#expression))
                }
                Child::Text(text) => {
                    return Err(Error::new_spanned(
                        text,
                        format!("`{}` cannot contain bare text; use `<Text>`", self.tag),
                    ))
                }
            };
            output = quote!(#output.child(#child));
        }
        Ok(output)
    }

    fn expand(&self) -> Result<TokenStream2> {
        match self.tag.to_string().as_str() {
            "RawView" => {
                self.reject_unknown_props(&["style", "background", "corner_radius"])?;
                let style = self.required_prop("style")?;
                let mut output = quote!(::creamui_widgets::raw::RawView::new(#style));
                if let Some(background) = self.prop("background")? {
                    output = quote!(#output.background(#background));
                }
                if let Some(radius) = self.prop("corner_radius")? {
                    output = quote!(#output.corner_radius(#radius));
                }
                self.container_children(output)
            }
            "View" => {
                self.reject_unknown_props(&["theme", "style"])?;
                let theme = self.required_prop("theme")?;
                let style = self.required_prop("style")?;
                self.container_children(
                    quote!(::creamui_widgets::themed::View::new(#theme, #style)),
                )
            }
            "ScrollView" => {
                self.reject_unknown_props(&["theme", "style", "scroll_y", "on_scroll"])?;
                let theme = self.required_prop("theme")?;
                let style = self.required_prop("style")?;
                let scroll_y = self.required_prop("scroll_y")?;
                let on_scroll = self.required_prop("on_scroll")?;
                self.container_children(quote!(::creamui_widgets::themed::ScrollView::new(#theme, #style, #scroll_y, #on_scroll)))
            }
            "Text" => {
                self.reject_unknown_props(&["theme", "font_size", "secondary"])?;
                let theme = self.required_prop("theme")?;
                let text = self.text_child()?;
                let mut output = if let Some(secondary) = self.prop("secondary")? {
                    quote!(if #secondary { ::creamui_widgets::themed::Text::secondary(#theme, #text) } else { ::creamui_widgets::themed::Text::new(#theme, #text) })
                } else {
                    quote!(::creamui_widgets::themed::Text::new(#theme, #text))
                };
                if let Some(font_size) = self.prop("font_size")? {
                    output = quote!(#output.font_size(#font_size));
                }
                Ok(output)
            }
            "Button" => {
                self.reject_unknown_props(&["theme", "on_click"])?;
                let theme = self.required_prop("theme")?;
                let on_click = self.required_prop("on_click")?;
                let label = self.text_child()?;
                Ok(quote!(::creamui_widgets::themed::Button::new(#theme, #label, #on_click)))
            }
            "Checkbox" => {
                self.reject_unknown_props(&["theme", "checked", "on_click"])?;
                if !self.children.is_empty() {
                    return Err(Error::new_spanned(
                        &self.tag,
                        "`Checkbox` cannot have children",
                    ));
                }
                let theme = self.required_prop("theme")?;
                let checked = self.required_prop("checked")?;
                let on_click = self.required_prop("on_click")?;
                Ok(quote!(::creamui_widgets::themed::Checkbox::new(#theme, #checked, #on_click)))
            }
            "TextInput" => {
                self.reject_unknown_props(&[
                    "theme",
                    "value",
                    "on_change",
                    "style",
                    "placeholder",
                ])?;
                if !self.children.is_empty() {
                    return Err(Error::new_spanned(
                        &self.tag,
                        "`TextInput` cannot have children",
                    ));
                }
                let theme = self.required_prop("theme")?;
                let value = self.required_prop("value")?;
                let on_change = self.required_prop("on_change")?;
                let mut output = if let Some(style) = self.prop("style")? {
                    quote!(::creamui_widgets::themed::TextInput::with_style(#theme, #style, #value, #on_change))
                } else {
                    quote!(::creamui_widgets::themed::TextInput::new(#theme, #value, #on_change))
                };
                if let Some(placeholder) = self.prop("placeholder")? {
                    output = quote!(#output.placeholder(#theme, #placeholder));
                }
                Ok(output)
            }
            "Slider" => {
                self.reject_unknown_props(&["theme", "value", "on_change", "style"])?;
                if !self.children.is_empty() {
                    return Err(Error::new_spanned(
                        &self.tag,
                        "`Slider` cannot have children",
                    ));
                }
                let theme = self.required_prop("theme")?;
                let value = self.required_prop("value")?;
                let on_change = self.required_prop("on_change")?;
                if let Some(style) = self.prop("style")? {
                    Ok(
                        quote!(::creamui_widgets::themed::Slider::with_style(#theme, #style, #value, #on_change)),
                    )
                } else {
                    Ok(quote!(::creamui_widgets::themed::Slider::new(#theme, #value, #on_change)))
                }
            }
            _ => self.expand_user_component(),
        }
    }

    fn expand_user_component(&self) -> Result<TokenStream2> {
        let tag = &self.tag;
        let children = self
            .children
            .iter()
            .map(|child| match child {
                Child::Element(element) => {
                    let expanded = element.expand()?;
                    Ok(quote!(::creamui_jsx::IntoWidget::into_widget(#expanded)))
                }
                Child::Expression(expression) => {
                    Ok(quote!(::creamui_jsx::IntoWidget::into_widget(#expression)))
                }
                Child::Text(text) => Err(Error::new_spanned(
                    text,
                    "custom components cannot contain bare text; use `<Text>`",
                )),
            })
            .collect::<Result<Vec<_>>>()?;
        if self.attributes.is_empty() {
            return if children.is_empty() {
                Ok(quote!(#tag()))
            } else {
                let props_name = format_ident!("{}Props", tag);
                Ok(quote!(#tag(#props_name { children: vec![ #( #children, )* ] })))
            };
        }
        if self.attributes.len() == 1 && self.attributes[0].name == "props" {
            if !children.is_empty() {
                return Err(Error::new_spanned(
                    &self.tag,
                    "`props` cannot be combined with JSX children; put `children` in the explicit props value",
                ));
            }
            let props = &self.attributes[0].value;
            return Ok(quote!(#tag(#props)));
        }
        if self
            .attributes
            .iter()
            .any(|attribute| attribute.name == "props")
        {
            return Err(Error::new_spanned(
                &self.tag,
                "`props` cannot be combined with named component props",
            ));
        }
        let props_name = format_ident!("{}Props", tag);
        let fields = self.attributes.iter().map(|attribute| {
            let name = &attribute.name;
            let value = &attribute.value;
            quote!(#name: #value)
        });
        if children.is_empty() {
            Ok(quote!(#tag(#props_name { #( #fields, )* })))
        } else {
            Ok(quote!(#tag(#props_name { #( #fields, )* children: vec![ #( #children, )* ] })))
        }
    }
}
