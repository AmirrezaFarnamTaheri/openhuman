pub fn collect_provider_ca_certs<'a>(
    providers: impl IntoIterator<Item = (&'a str, Option<&'a str>)>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let mut certs = std::collections::HashMap::new();
    for (slug, pem) in providers {
        let Some(pem) = pem else { continue };
        if !pem.is_empty() {
            crate::util::tls::parse_ca_bundle(pem)?;
        }
        certs.insert(slug.trim().to_string(), pem.to_string());
    }
    Ok(certs)
}
