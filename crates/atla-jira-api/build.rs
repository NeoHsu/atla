use progenitor::{GenerationSettings, Generator, InterfaceStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let src = "../../specs/jira-v3-partial.json";
    println!("cargo:rerun-if-changed={src}");

    let file = std::fs::File::open(src)?;
    let spec: openapiv3::OpenAPI = serde_json::from_reader(file)?;

    let mut settings = GenerationSettings::default();
    settings
        .with_interface(InterfaceStyle::Builder)
        .with_derive("PartialEq");

    let mut generator = Generator::new(&settings);
    let tokens = generator.generate_tokens(&spec).map_err(|error| {
        std::io::Error::other(format!("progenitor generation failed: {error:?}"))
    })?;
    let ast = syn::parse2(tokens)?;
    let content = prettyplease::unparse(&ast);

    let out = std::path::Path::new(&std::env::var("OUT_DIR")?).join("codegen.rs");
    std::fs::write(out, content)?;
    Ok(())
}
