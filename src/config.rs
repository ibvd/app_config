use std::fs;
use shellexpand::tilde;

use crate::providers::{Provider, AWSConf, MockConf};
use crate::hooks::{Hook, TemplateConf, FileConf, RawConf, CommandConf};

type TResult<T> = Result<T, toml::de::Error>;


// This is a bit hard to read, but here is the deal.
// There is a BTree in <maps> that contains the structure of the config file
// There is a Vec in <hooks> where we store our final structs
// This macro will loop over every hook in <maps>, convert the hook into a struct
// and push the result into <hooks>. 
#[macro_export]
macro_rules! parse_hooks {
    ( $( $maps:expr, $hooks:expr, $($section:expr, $conf:ty),+)? ) => {
        { $(
    for hook_section in $maps["hooks"].as_table().unwrap().keys() {
        $(
        if hook_section.as_str() == $section {
            let conf: TResult<$conf> = $maps["hooks"][$section]
                .clone().try_into();
            match conf {
                Err(e) => config_err(&e, $section),
                Ok(conf) => {
                    let x = conf.convert();
                    $hooks.push( Box::new(x) );
                },
            }
        }
        )+
    }
        )? }
    };
}


// Like for parse_hooks above, but instead we only want one provider. So it is 
// an if / else if / else if ... / chain.  Erroring out if nothing matches.
// There is a BTree in <maps> that contains the structure of the config file
// This macro will check for each provider in <maps>, convert the provider into a 
// struct and save the result into <provider>. 
#[macro_export]
macro_rules! parse_providers {
    ( $( $maps:expr, $provider_type:expr, $provider:expr, 
                                    $($section:expr, $conf:ty),+)? ) => {
        { $(
        if ! true { }
        $(
        // AWS 
        else if $provider_type.as_str() == $section {
            let conf: TResult<$conf> = $maps["providers"][$section]
                                                    .clone().try_into();
            // Pretty print any parsing errors
            if let Err(e) = &conf { config_err(&e, $section); }

            let x = conf.unwrap().convert();
            $provider = Box::new(x);
        } 
        )+
        // If no valid provider found, panic with an error
        else {
            eprintln!("Error, no valid providers found");
            std::process::exit(exitcode::CONFIG);
        }
        )? }
    };
}


/// Config:
/// Parse toml config file and validate all the parameters
#[derive(Debug)]
pub struct Config {
    pub provider: Box<dyn Provider>,
    pub hooks: Vec<Box<dyn Hook>>,
}

impl Config {
    /// Read toml formatted config file  located @ <path>, 
    /// and parse it into a Config struct.  
    /// Will panic if it can not locate or parse the file.
    pub fn from_file(path: &str) -> Config {

        let expanded_path = String::from(tilde(&path));
        let file_contents: String = match fs::read_to_string(expanded_path) {
            Ok(file_contents) => file_contents,
            Err(e) => {
                eprintln!("Could not open {}: {}", path, e);
                std::process::exit(exitcode::OSFILE);
            },
        };
    
        let toml_maps: toml::Value = match toml::from_str(&file_contents) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Could not parse {}: {}", path, e);
                std::process::exit(exitcode::CONFIG);
            },
        };

        // Extract provider from config file
        let p: Box<dyn Provider> = Config::get_provider(&toml_maps);
        
        // Extract hooks from config file
        let h: Vec<Box<dyn Hook>> = Config::get_hooks(&toml_maps);
        
        Config { provider: p, hooks: h }
    }


    /// Parse the config file looking for one and only one backend provider
    /// Will panic on any errors. 
    fn get_provider(maps: &toml::Value) -> Box<dyn Provider> {
        
        // Validate Providers are present
        if ! maps.as_table().unwrap().contains_key("providers") {
            eprintln!("Error, configuation must include a backend provider");
            std::process::exit(exitcode::CONFIG);
        }
    
        if maps["providers"].as_table().unwrap().len() != 1 {
            eprintln!("Error, configuation must include only one backend provider");
            std::process::exit(exitcode::CONFIG);
        }
    
        let mut provider: Box<dyn Provider>;
        // This is done just to let us use a macro to parse the providers. Rust
        // gets confused.  We will panic before this provider ever gets further.
        provider = Box::new(MockConf{data: "".to_string()}.convert());
    
        // Since we know we have just one provider key, let's get it
        let provider_type = maps["providers"].as_table().unwrap()
                                             .keys().last().unwrap();

        // This macro will find the configured provider in <maps> and instantiate
        // the provider struct in <provider>. It will panic if no provider is found
        // or if there is a parsing error in the provider section.
        parse_providers!(maps, provider_type, provider, 
                "mock", MockConf,
                "aws",  AWSConf
                );

        provider
    }

    /// Parse the config file looking for hooks
    /// The order in the vec will be the same as specified in the config file
    /// Will panic on any errors. 
    fn get_hooks(maps: &toml::Value) -> Vec<Box<dyn Hook>> {

        let mut hooks: Vec<Box<dyn Hook>> = Vec::new();

        // Validate there are at least some hooks in the config file
        if ! maps.as_table().unwrap().contains_key("hooks") {
            return hooks;
        }

        // This macro will instantiate a struct for each hook found in 
        // maps["hooks"], and push that hook into the 'hooks' vector
        parse_hooks!(maps, hooks, 
                "template", TemplateConf,
                "file",     FileConf,
                "raw",      RawConf,
                "command",  CommandConf
                );

        hooks
    }
}

fn config_err(e: &toml::de::Error, section: &str) {
    eprintln!("Could not parse {} config: {:#?}", section, e);
    std::process::exit(exitcode::CONFIG);
}






#[cfg(test)]
mod test {
    use super::*;
    use crate::providers::{AWS};
    use crate::hooks::{Hook, Template, File, Command};
    use crate::hooks::template::DataType;

    fn gen_full_config() -> String {
"[providers.aws]
application = \"myApp\"
environment = \"dev\"
configuration = \"myConf\"
client_id = \"42\"

[hooks.template]
file = \"./tests/test_template.tmpl\"
source_type = \"yaml\"

[hooks.file]
outfile = \"raw_output.txt\"

[hooks.command]
command = \"echo\"
pipe_data = true
".to_string()
    }

    fn gen_min_config() -> String {
"[providers.aws]
application = \"myApp\"
environment = \"dev\"
configuration = \"myConf\"
client_id = \"42\"".to_string()
    }

    fn gen_aws_struct() -> AWS {
        AWS::new(&"myApp", &"dev", &"myConf", &"42", &None)
    }

    fn gen_template_struct() -> Template {
        Template::new( 
            &String::from("{{#each hosts}}
[Peer]
EndPoint = {{this.name}}
PublicKey = {{this.public_key}}
{{/each}}
"),
            DataType::YAML, 
            None)
    } 

    fn gen_file_struct() -> File {
        File::new(&"raw_output.txt")
    }

    fn gen_command_struct() -> Command {
        Command::new(&"echo", true)
    }

    #[test]
    // We can not compare structs directly since they are hidden behind a 
    // dynamic trait, The compiler has no idea what struct will be there at 
    // compile time.  So the best we can do is print them and compare the 
    // output strings from the Debug trait.
    fn test_get_provider() {
        let config_str = gen_full_config();
        let tml: toml::Value = toml::from_str(&config_str).unwrap();
        let expected_str = format!("{:?}", gen_aws_struct() );
        let provider_str = format!("{:?}", Config::get_provider(&tml) );
        assert_eq!(expected_str, provider_str);
    }

    #[test]
    fn test_get_hooks() {
        let config_str = gen_full_config();
        let tml: toml::Value = toml::from_str(&config_str).unwrap();
        let h = Config::get_hooks(&tml);
        let hook_str = format!("{:?}", h );
        let expected: Vec<Box<dyn Hook>> = vec![
                            Box::new(gen_template_struct()), 
                            Box::new(gen_file_struct()),
                            Box::new(gen_command_struct()),
        ];

        let expected_str = format!("{:?}", expected);
        assert_eq!(hook_str, expected_str);
    }

    #[test]
    fn test_get_empty_hooks() {
        let config_str = gen_min_config();
        let tml: toml::Value = toml::from_str(&config_str).unwrap();
        let h = Config::get_hooks(&tml);
        let hook_str = format!("{:?}", h );

        let expected_str = format!("[]");
        assert_eq!(expected_str, hook_str);
    }
}
