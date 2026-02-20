use crate::config::model::AppConfig;
use crate::error::AppError;

pub fn validate(config: &AppConfig) -> Result<(), AppError> {
    let mut errors: Vec<String> = Vec::new();

    if config.streams.is_empty() {
        errors.push("at least one [[streams]] must be defined".into());
    }

    for (i, stream) in config.streams.iter().enumerate() {
        let prefix = format!("streams[{}] ('{}')", i, stream.name);

        if stream.name.is_empty() {
            errors.push(format!("{prefix}: name cannot be empty"));
        }

        // Validate format type
        let valid_formats = [
            "syslog_rfc5424",
            "syslog_rfc3164",
            "cef",
            "leef",
            "clf",
            "json",
            "java_log4j",
            "java_logback",
            "template",
            "script",
        ];
        if !valid_formats.contains(&stream.format.format_type.as_str()) {
            errors.push(format!(
                "{prefix}: unknown format type '{}', valid types: {}",
                stream.format.format_type,
                valid_formats.join(", ")
            ));
        }

        // Validate output type
        let valid_outputs = ["stdout", "file", "tcp", "udp", "http"];
        if !valid_outputs.contains(&stream.output.output_type.as_str()) {
            errors.push(format!(
                "{prefix}: unknown output type '{}', valid types: {}",
                stream.output.output_type,
                valid_outputs.join(", ")
            ));
        }

        // Validate output-specific required fields
        match stream.output.output_type.as_str() {
            "file" => {
                if stream.output.path.is_none() {
                    errors.push(format!("{prefix}: output type 'file' requires 'path'"));
                }
            }
            "tcp" | "udp" => {
                if stream.output.host.is_none() {
                    errors.push(format!(
                        "{prefix}: output type '{}' requires 'host'",
                        stream.output.output_type
                    ));
                }
                if stream.output.port.is_none() {
                    errors.push(format!(
                        "{prefix}: output type '{}' requires 'port'",
                        stream.output.output_type
                    ));
                }
            }
            "http" => {
                if stream.output.url.is_none() {
                    errors.push(format!("{prefix}: output type 'http' requires 'url'"));
                }
            }
            _ => {}
        }

        // Validate script format
        if stream.format.format_type == "script" {
            if stream.format.script_file.is_none() && stream.format.script_inline.is_none() {
                errors.push(format!(
                    "{prefix}: format type 'script' requires 'script_file' or 'script_inline'"
                ));
            }
            if let Some(max_ops) = stream.format.max_operations {
                if max_ops == 0 {
                    errors.push(format!("{prefix}: max_operations must be > 0"));
                }
            }
        }

        // Validate rate
        let eps = stream
            .rate
            .as_ref()
            .and_then(|r| r.eps)
            .or(config.defaults.as_ref().and_then(|d| d.rate));

        if let Some(eps_val) = eps {
            if eps_val <= 0.0 {
                errors.push(format!("{prefix}: eps must be positive, got {eps_val}"));
            }
        } else if stream.rate.as_ref().and_then(|r| r.wave.as_ref()).is_none() {
            errors.push(format!(
                "{prefix}: rate.eps is required (or set defaults.rate, or use a wave)"
            ));
        }

        // Validate wave if present
        if let Some(wave) = stream.rate.as_ref().and_then(|r| r.wave.as_ref()) {
            let valid_shapes = ["sine", "sawtooth", "square"];
            if !valid_shapes.contains(&wave.shape.as_str()) {
                errors.push(format!(
                    "{prefix}: unknown wave shape '{}', valid: {}",
                    wave.shape,
                    valid_shapes.join(", ")
                ));
            }
            if wave.min < 0.0 {
                errors.push(format!("{prefix}: wave.min must be >= 0"));
            }
            if wave.max < wave.min {
                errors.push(format!("{prefix}: wave.max must be >= wave.min"));
            }
            if wave.period_secs <= 0.0 {
                errors.push(format!("{prefix}: wave.period_secs must be positive"));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Config(errors.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::AppConfig;

    #[test]
    fn valid_minimal_config() {
        let toml = r#"
            [[streams]]
            name = "test"

            [streams.format]
            type = "syslog_rfc5424"

            [streams.output]
            type = "stdout"

            [streams.rate]
            eps = 10.0
        "#;
        let config = AppConfig::from_toml(toml).unwrap();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn rejects_empty_streams() {
        let toml = r#"
            streams = []
        "#;
        let config = AppConfig::from_toml(toml).unwrap();
        let err = validate(&config).unwrap_err();
        assert!(err.to_string().contains("at least one"));
    }

    #[test]
    fn rejects_unknown_format() {
        let toml = r#"
            [[streams]]
            name = "test"

            [streams.format]
            type = "invalid_format"

            [streams.output]
            type = "stdout"

            [streams.rate]
            eps = 10.0
        "#;
        let config = AppConfig::from_toml(toml).unwrap();
        let err = validate(&config).unwrap_err();
        assert!(err.to_string().contains("unknown format type"));
    }

    #[test]
    fn rejects_file_without_path() {
        let toml = r#"
            [[streams]]
            name = "test"

            [streams.format]
            type = "syslog_rfc5424"

            [streams.output]
            type = "file"

            [streams.rate]
            eps = 10.0
        "#;
        let config = AppConfig::from_toml(toml).unwrap();
        let err = validate(&config).unwrap_err();
        assert!(err.to_string().contains("requires 'path'"));
    }

    #[test]
    fn rejects_negative_eps() {
        let toml = r#"
            [[streams]]
            name = "test"

            [streams.format]
            type = "syslog_rfc5424"

            [streams.output]
            type = "stdout"

            [streams.rate]
            eps = -5.0
        "#;
        let config = AppConfig::from_toml(toml).unwrap();
        let err = validate(&config).unwrap_err();
        assert!(err.to_string().contains("must be positive"));
    }
}
