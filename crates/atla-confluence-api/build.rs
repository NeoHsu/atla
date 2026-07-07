use progenitor::{GenerationSettings, Generator, InterfaceStyle};

fn main() {
    let src = "../../specs/confluence-v2-partial.json";
    println!("cargo:rerun-if-changed={src}");

    let file = std::fs::File::open(src).unwrap();
    let spec: openapiv3::OpenAPI = serde_json::from_reader(file).unwrap();

    let mut settings = GenerationSettings::default();
    settings
        .with_interface(InterfaceStyle::Builder)
        .with_derive("PartialEq");

    let mut generator = Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec).unwrap_or_else(|e| {
        panic!("progenitor generation failed: {e:?}");
    });
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    let out = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("codegen.rs");
    std::fs::write(out, content).unwrap();
}
