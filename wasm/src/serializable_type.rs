// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Macro to define a struct, with a corresponding TypeScript type,
/// which represents a JavaScript data object.
///
/// A struct defined via this macro is serializable/deserializable so it can
/// be passed across the WASM boundary as an argument or a return value.
///
/// The TypeScript type defined alongside the struct is included in
/// the .d.ts file generated by wasm-bindgen.
///
/// Optionally, an identifier can be provided for the TypeScript type to
/// be referenced by Rust code. This makes it so that the type can be
/// used in method signatures instead of `JsValue`. The generated .d.ts
/// file will then contain the corresponding TypeScript type in the
/// method signatures instead of `any`.
///
/// When the TypeScript type identifier is provided to the macro, the
/// `From` trait is implemented for conversion between the TypeScript type
/// (`JsValue`) and the Rust struct using `serde_wasm_bindgen`.
///
/// # Examples
///
/// ```
/// serializable_type! {
///     pub struct Hover {
///         pub contents: String,
///         pub span: Span,
///     },
///     r#"export interface IHover {
///         contents: string;
///         span: ISpan
///     }"#,
///     Hover,
///     IHover,
///     "IHover"
/// }
/// ```
///
/// This type can now be used as the argument or return type
/// in a Rust method exported by wasm-bindgen. Converting between
/// the TypeScript type and the Rust struct are is trivial using
/// `into()` since the `From` trait implementation is generated
/// by the macro:
///
/// ```
/// pub fn get_hover(&self) -> Option<IHover> {
///     let hover = Hover { contents: get_contents(), span: get_span() };
///     hover.into()
/// }
/// ```
///
/// The generated TypeScript method signature would be:
///
/// ```
/// get_hover(): IHover | undefined;
/// ```
///
/// The last three arguments into the macro can be omitted
/// if the TypeScript type doesn't need to be directly referenced from
/// Rust code, i.e. if the struct isn't meant to be used as the
/// in method signatures.
///
/// The following data type is serializable, and can be used within
/// other serializable structs, but since we omitted the identifier
/// arguments, it cannot be referenced by its TypeScript type name in
/// Rust code. Therefore it cannot be used directly in method
/// signatures.
///
/// ```
/// serializable_type! {
///     pub struct Span {
///         pub start: u32,
///         pub end: u32,
///     },
///     r#"export interface ISpan {
///         start: number;
///         end: number;
///     }"#
/// }
/// ```
///
macro_rules! serializable_type {
    ($struct: item, $typescript: literal) => {
        #[derive(Debug, Serialize, Deserialize)]
        #[allow(non_snake_case)] // These types propagate to JS which expects camelCase
        $struct

        // TypeScript type definition that will be included in the generated .d.ts file.
        // This name will be shadowed every time we redeclare it but it doesn't matter
        // for the purposes here.
        #[wasm_bindgen(typescript_custom_section)]
        const TYPESCRIPT_CUSTOM_SECTION: &'static str = $typescript;
    };

    ($struct: item, $typescript: literal, $struct_ident: ident, $typescript_type_ident:ident, $typescript_type_name: literal) => {
        #[derive(Serialize, Deserialize)]
        #[allow(non_snake_case)] // These types propagate to JS which expects camelCase
        $struct

        // TypeScript type definition that will be included in the generated .d.ts file.
        // This name will be shadowed every time we redeclare it but it doesn't matter
        // for the purposes here.
        #[wasm_bindgen(typescript_custom_section)]
        const TYPESCRIPT_CUSTOM_SECTION: &'static str = $typescript;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(typescript_type = $typescript_type_name)]
            #[doc=$typescript]
            pub type $typescript_type_ident;
        }

        impl From<$typescript_type_ident> for $struct_ident {
            fn from(js_val: $typescript_type_ident) -> Self {
                serde_wasm_bindgen::from_value(js_val.into()).unwrap()
            }
        }

        impl From<$struct_ident> for $typescript_type_ident {
            fn from(val: $struct_ident) -> Self {
                serde_wasm_bindgen::to_value(&val).unwrap().into()
            }
        }
    };
}

pub(crate) use serializable_type;
