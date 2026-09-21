#![allow(unused_imports, clippy::module_inception)]

#[allow(unused_imports, clippy::wildcard_imports)]
use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub(super) fn endpoint_and_timeout_are_validated_before_transport() {
        let args = Cli::try_parse_from(["oscal-cli", "--endpoint", "", "--timeout-secs", "0"])
            .expect("clap should parse values for domain validation");
        let error = AppConfig::from_args(&args).expect_err("invalid configuration must fail");
        assert!(error.to_string().contains("endpoint"));
    }

    #[test]
    pub(super) fn m_tls_identity_requires_both_files() {
        let args = Cli::try_parse_from([
            "oscal-cli",
            "--endpoint",
            "https://observer.example",
            "--client-cert",
            "cert.pem",
        ])
        .expect("clap should parse values for domain validation");
        let error = AppConfig::from_args(&args).expect_err("partial identity must fail");
        assert!(error.to_string().contains("client-cert and client-key"));
    }

    #[test]
    pub(super) fn token_file_is_loaded_without_retaining_newlines() {
        let path = std::env::temp_dir().join(format!("oscal-cli-token-{}", std::process::id()));
        fs::write(&path, " bearer-token\n").expect("token fixture should be written");
        let args = Cli::try_parse_from([
            "oscal-cli",
            "--token-file",
            path.to_str().expect("token path should be valid"),
        ])
        .expect("clap should parse token file");

        let config = AppConfig::from_args(&args).expect("token file should load");

        assert_eq!(config.token.as_deref(), Some("bearer-token"));
        fs::remove_file(path).expect("token fixture should be removed");
    }

    #[test]
    pub(super) fn empty_token_is_rejected_without_echoing_secret_material() {
        let args = Cli::try_parse_from(["oscal-cli", "--token", "   "])
            .expect("clap should parse token values");

        let error = AppConfig::from_args(&args).expect_err("empty token must fail");

        assert_eq!(
            error.to_string(),
            "configuration error: token must not be empty"
        );
    }

    #[test]
    pub(super) fn search_accepts_an_opaque_page_token() {
        let args = Cli::try_parse_from([
            "oscal-cli",
            "search",
            "catalog",
            "--page-token",
            "opaque-page-token",
        ])
        .expect("search page token should parse");

        let Command::Search(search) = args.command.expect("search command should be present")
        else {
            panic!("expected search command");
        };
        assert_eq!(search.page_token, "opaque-page-token");
    }
}
