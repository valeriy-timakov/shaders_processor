use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};
use regex::Regex;
use proc_macro2::Span;
use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;

#[proc_macro]
pub fn include_shader(input: TokenStream) -> TokenStream {
    // 1) Розібрати ім'я шейдера
    let shader_name_lit = parse_macro_input!(input as LitStr);
    let shader_name = shader_name_lit.value();

    // 2) Стандартний шлях від кореня crate до src/shaders
    let mut base = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    base.push("src");
    base.push("shaders");

    // 3) Стягнути всі куски: спочатку включені, потім головний
    let mut seen = HashSet::new();
    let mut parts = Vec::new();

    // Завантажуємо та чистимо файл із включеннями
    fn load_and_clean(
        name: &str,
        dir: &PathBuf,
        seen: &mut HashSet<String>,
        parts: &mut Vec<String>,
    ) {
        // Уникнемо циклічного include
        if !seen.insert(name.to_string()) {
            return;
        }

        let mut path = dir.clone();
        path.push(format!("{}.wgsl", name));
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Can't read shader `{}`", path.display()));

        // regex для директив
        let re = Regex::new(r#"#include\s+"([^"]+)""#).unwrap();

        // Знайти всі include і рекурсивно обробити
        for cap in re.captures_iter(&text) {
            let inc = &cap[1];
            load_and_clean(inc, dir, seen, parts);
        }

        // Видалити усі рядки з include
        let cleaned = re.replace_all(&text, "");

        // Додати до списку кусок
        parts.push(cleaned.into_owned());
    }

    // Починаємо з головного шейдера
    load_and_clean(&shader_name, &base, &mut seen, &mut parts);

    // Об'єднуємо усі pieces в один літерал
    let full = parts.join("\n");
    let lit = LitStr::new(&full, Span::call_site());

    TokenStream::from(quote! {
        #lit
    })
}
