use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprMethodCall, ExprPath, Stmt, Token};

#[proc_macro]
pub fn pincers(input: TokenStream) -> TokenStream {
    let block = parse_macro_input!(input as syn::Block);
    // Collect widget descriptions: (type_name, vec[(field_ident, expr)])
    let mut widgets: Vec<(syn::Ident, Vec<(syn::Ident, syn::Expr)>)> = Vec::new();

    for stmt in block.stmts {
        let expr = match stmt {
            Stmt::Expr(expr, None) | Stmt::Expr(expr, Some(_)) => expr,
            _ => {
                return syn::Error::new_spanned(stmt, "expected expression ending with ';'")
                    .to_compile_error()
                    .into();
            }
        };

        let (ty, fields) = match parse_chain(expr) {
            Ok(res) => res,
            Err(e) => return e.to_compile_error().into(),
        };
        widgets.push((ty, fields));
    }

    // Determine unique widget types and collect fields (ident -> type tokens)
    use std::collections::HashMap;
    let mut type_fields: HashMap<syn::Ident, HashMap<syn::Ident, proc_macro2::TokenStream>> =
        HashMap::new();

    // Known method -> (field name, type tokens)
    let mut known_methods = HashMap::new();
    known_methods.insert(
        syn::Ident::new("text", proc_macro2::Span::call_site()),
        ("text", quote! { String }),
    );
    known_methods.insert(
        syn::Ident::new("padding", proc_macro2::Span::call_site()),
        ("padding", quote! { u8 }),
    );
    known_methods.insert(
        syn::Ident::new("visible", proc_macro2::Span::call_site()),
        ("visible", quote! { bool }),
    );
    known_methods.insert(
        syn::Ident::new("id", proc_macro2::Span::call_site()),
        ("id", quote! { u32 }),
    );

    for (ty, fields) in &widgets {
        let entry = type_fields.entry(ty.clone()).or_default();
        for (method, expr) in fields {
            let (field_name, ty_tokens) = known_methods.get(method).map_or_else(
                || {
                    // Infer type from literal expression
                    let ty_tokens = infer_type(expr.clone());
                    (method.to_string(), ty_tokens)
                },
                |(fname, ty)| (fname.to_string(), ty.clone()),
            );
            let field_ident = syn::Ident::new(&field_name, method.span());
            entry.insert(field_ident, ty_tokens);
        }
    }

    // Generate struct definitions for each widget type
    let mut struct_items = Vec::new();
    let mut enum_variants = Vec::new();
    let mut builder_methods = Vec::new();

    for (ty, fields) in &type_fields {
        let struct_name = ty;
        let mut struct_fields = Vec::new();
        for (fname, fty) in fields {
            struct_fields.push(quote! { #fname: #fty });
        }
        struct_items.push(quote! {
            #[derive(Default)]
            struct #struct_name {
                #(#struct_fields),*
            }
        });

        let variant_name = ty;
        enum_variants.push(quote! { #variant_name(#struct_name) });

        // Builder method for this widget type (takes self by value)
        let method_name =
            syn::Ident::new(&format!("add_{}", ty.to_string().to_lowercase()), ty.span());
        builder_methods.push(quote! {
            fn #method_name<F>(self, f: F) -> Self
            where
                F: FnOnce(&mut #struct_name),
            {
                let mut widget = #struct_name::default();
                f(&mut widget);
                let mut new_vec = self.vec;
                new_vec.push(Widget::#variant_name(widget));
                Self { vec: new_vec }
            }
        });
    }

    // Generate the const MAIN_WINDOW using the builder
    let mut build_stmts = Vec::new();
    build_stmts.push(quote! { let mut win = PincersWindow::new(); });

    for (ty, fields) in &widgets {
        let struct_name = ty;
        // Collect field idents and exprs for this widget
        let mut idents = Vec::new();
        let mut exprs = Vec::new();
        for (method, expr) in fields {
            let field_name = known_methods
                .get(method)
                .map(|(f, _)| f.to_string())
                .unwrap_or_else(|| method.to_string());
            let ident = syn::Ident::new(&field_name, method.span());
            idents.push(ident);
            exprs.push(expr.clone());
        }
        // Build a closure that sets fields on a temporary struct
        let struct_name_clone = struct_name.clone();
        let init = quote! {
            |tmp: &mut #struct_name_clone| {
                #(
                    #idents = #exprs;
                )*
                tmp
            }
        };
        // Determine which builder method to call
        let builder_ident =
            syn::Ident::new(&format!("add_{}", ty.to_string().to_lowercase()), ty.span());
        build_stmts.push(quote! {
            win = win.#builder_ident(#init);
        });
    }
    build_stmts.push(quote! { win });

    let expanded = quote! {
        #(#struct_items)*

        enum Widget {
            #(#enum_variants),*
        }

        struct PincersWindow {
            vec: Vec<Widget>,
        }

        impl PincersWindow {
            fn new() -> Self {
                Self { vec: Vec::new() }
            }

            #(#builder_methods)*

            fn render(&self, _display: &mut dyn core::any::Any) -> Option<u32> {
                // Stub implementation – real implementation would interact with display
                None
            }
        }

        const MAIN_WINDOW: PincersWindow = {
            #(#build_stmts)*
        };
    };

    TokenStream::from(expanded)
}

fn parse_chain(mut expr: Expr) -> syn::Result<(syn::Ident, Vec<(syn::Ident, Expr)>)> {
    let mut methods = Vec::new();
    loop {
        match expr {
            Expr::Path(p) => {
                let path = p.path;
                if path.segments.len() != 1 {
                    return Err(syn::Error::new_spanned(
                        path,
                        "expected single identifier as base type",
                    ));
                }
                let ident = path.segments[0].ident.clone();
                return Ok((ident, methods));
            }
            Expr::MethodCall(mc) => {
                let method = mc.method.clone();
                let args = mc.args.iter().cloned().collect::<Vec<_>>();
                if args.len() != 1 {
                    return Err(syn::Error::new_spanned(
                        mc,
                        "expected single argument in method call",
                    ));
                }
                let arg = args.into_iter().next().unwrap();
                methods.push((method, arg));
                expr = *mc.receiver;
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    expr,
                    "expected method chain or identifier",
                ));
            }
        }
    }
}

/// Infer a Rust type from a literal expression.
fn infer_type(expr: Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Lit(lit) => {
            let lit = lit.lit;
            if let syn::Lit::Str(_) = lit {
                quote! { String }
            } else if let syn::Lit::Int(_) = lit {
                quote! { u32 }
            } else if let syn::Lit::Bool(_) = lit {
                quote! { bool }
            } else {
                // fallback to string
                quote! { String }
            }
        }
        _ => quote! { String },
    }
}
