use crate::hooks::Hook;
use serde_derive::Deserialize;
use eyre::Result;

use shellexpand::tilde;
use std::fs;
use std::io::prelude::*;

use handlebars::{Handlebars, RenderContext, Helper, Context, JsonRender, 
                 HelperResult, Output };
use crate::providers::param_store::get_params;


// // // // // // // // // Handle Configuraion // // // // // // // //

// TemplateConf will store the user's input from the configuration file
// and then let us instantiate a Template struct
#[derive(Debug, Deserialize)]
#[serde(rename = "template")]
pub struct TemplateConf {
    file: String,
    source_type: DataType,
    out_file: Option<String>,
}

impl TemplateConf {
    pub fn convert(&self) -> Template {
        // Read in the template from the provided file.
        let expanded_path = String::from(tilde(&self.file));

        let file_contents: String = match fs::read_to_string(expanded_path) {
            Ok(file_contents) => file_contents,
            Err(e) => {
                eprintln!("Could not open {}: {}", &self.file, e);
                std::process::exit(exitcode::OSFILE);
            }
        };

        Template::new(
            &file_contents,
            self.source_type.clone(),
            self.out_file.clone(),
        )
    }
}


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    YAML,
    JSON,
    TOML,
}


// // // // // // // // // // // Hook // // // // // // // // // // //

/// The Template hook will take formatted data (yaml, toml, json) from the provider
/// and render it using a Handlebars template stored in <tpl>. If <out_file> is
/// ommited the template will be rendered to stdout. Else it will be saved to a file.
#[derive(Debug)]
pub struct Template {
    tpl: String,
    source_type: DataType,
    out_file: Option<String>,
}

impl Template {
    /// Create a new Template struct
    pub fn new(tpl: &str, source_type: DataType, out_file: Option<String>) -> Template {
        Template {
            tpl: tpl.to_string(),
            source_type,
            out_file,
        }
    }

    /// Render the template
    fn render(&self, data: &str) -> String {
        let transformed_data = Template::transform(&self.source_type, data);

        let mut hb = Handlebars::new();
        hb.register_helper("key", Box::new(key_helper));

        assert!(hb.register_template_string("tpl", self.tpl.clone()).is_ok());

        hb.render("tpl", &transformed_data).unwrap()
    }

    /// Source data from YAML, JSON or TOML and turn it all into a BTreeMap
    /// for use with Handlebars templates
    fn transform(source_type: &DataType, input_data: &str) -> serde_yaml::Value {
        match source_type {
            DataType::YAML => serde_yaml::from_str(input_data).unwrap(),
            DataType::JSON => serde_json::from_str(input_data).unwrap(),
            DataType::TOML => toml::from_str(input_data).unwrap(),
        }
    }
}

impl Hook for Template {
    /// Render the data and either print to stdout,
    /// or save the output to a file
    fn run(&self, data: &str) -> Result<()> {
        let rendered_data = &self.render(data);

        // If the user configured 'out_file', write the template there
        // Else print the rendered templete to stdout
        match &self.out_file {
            Some(file) => {
                let expanded_path = tilde(&file).to_string();

                match fs::File::create(expanded_path) {
                    Ok(mut file_handle) => 
                        file_handle.write_all(rendered_data.as_bytes())?,
                    Err(e) => {
                        eprintln!("Could not open {}: {}", file, e);
                        std::process::exit(exitcode::OSFILE);
                    }
                };
            }
            None => print!("{}", rendered_data),
        };
        Ok(())
    }
}


/// Handlebars helper function that will accept an AWS Parameter Store Key and
/// Return the result.   Assume in AWS Paramstore there is a key called "Hello"
/// with a value "World".  In the template we can write 
/// `Greetings: {{key "Hello"}}` and when rendered we see: `Greetings: World`
fn key_helper (
    h: &Helper, _: &Handlebars, _: &Context, _rc: &mut RenderContext, 
                                    out: &mut dyn Output) -> HelperResult {

    let ssm_key: String = h.param(0).unwrap().value().render();
    let value = match get_params(&ssm_key) {
        Ok(value) => value,
        Err(e) => return Err(handlebars::RenderError::new(format!("{:#?}", e))),
    };

    out.write(&value)?;
    Ok(())

}
    

// // // // // // // // // // // Tests // // // // // // // // // // //

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_yml_data() -> &'static str {
        "---
hosts:
  - name: host1
    public_key: xyz
  - name: host2
    public_key: abc"
    }

    fn gen_json_data() -> &'static str {
        "{
\"hosts\": [
    { \"name\": \"host1\", \"public_key\": \"xyz\" },
    { \"name\": \"host2\", \"public_key\": \"abc\" }
]
}"
    }

    fn gen_toml_data() -> &'static str {
        "
[hosts]

[hosts.1]
name = 'host1'
public_key = 'xyz'

[hosts.2]
name = 'host2'
public_key = 'abc'
"
    }

    fn gen_expected() -> &'static str {
        "
[Peer]
EndPoint = host1
PublicKey = xyz

[Peer]
EndPoint = host2
PublicKey = abc
"
    }

    fn gen_template() -> &'static str {
        "{{#each hosts}}
[Peer]
EndPoint = {{this.name}}
PublicKey = {{this.public_key}}
{{/each}}"
    }

    #[test]
    fn test_yaml_template() {
        let expected = gen_expected();
        let tpl = Template {
            tpl: gen_template().to_string(),
            // data: gen_yml_data().to_string(),
            source_type: DataType::YAML,
            out_file: None,
        };
        let res = tpl.render(gen_yml_data());

        assert_eq!(expected, res);
    }

    #[test]
    fn test_json_template() {
        let expected = gen_expected();
        let tpl = Template {
            tpl: gen_template().to_string(),
            // data: gen_json_data().to_string(),
            source_type: DataType::JSON,
            out_file: None,
        };
        let res = tpl.render(gen_json_data());

        assert_eq!(expected, res);
    }

    #[test]
    fn test_toml_template() {
        let expected = gen_expected();
        let tpl = Template {
            tpl: gen_template().to_string(),
            // data: gen_toml_data().to_string(),
            source_type: DataType::TOML,
            out_file: None,
        };
        let res = tpl.render(gen_toml_data());

        assert_eq!(expected, res);
    }
}
