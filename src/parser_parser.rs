/**
TODO: second round of parisng
not sure how to implement quite yet but going to try and get some tests written first and go from there...

hopefully refactoring will only be needed for the helper function

*/

use crate::ruby_parser::RubyFile;

pub fn parse(file: RubyFile, files: &Vec<RubyFile>) -> Result<RubyFile, String> {
    Ok(file)
}

#[cfg(test)]
mod second_parser {
    use std::vec;
    use pretty_assertions::assert_eq;

    use lib_ruby_parser::Parser;

    use crate::parser_parser::parse;
    use crate::ruby_parser::{parse_file, Controller, RubyFile, Method};

    fn helper(subject: &str, files: Vec<&str>) -> Result<RubyFile, String> {
        let key_file = parse_file(
            Parser::new(subject.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
        )?;
        println!("ran key_file");
        let mut data = Vec::new();

        for file in files {
            println!("doing file: {}", file);
            data.push(parse_file(
                Parser::new(file.as_bytes(), Default::default())
                    .do_parse()
                    .ast
                    .unwrap(),
            )?);
        }

        parse(key_file, &data)
    }

    #[ignore = "WIP"]
    #[test]
    fn basic_integration() {
        let main = "
        class VersionController < ApplicationController

            def version
                json(200, 'version', param[:cat])
            end
        end
      ";
        let files = vec![
            "class ApplicationConroller < ActionController::API
            include ConResponse
            
            before_action :check_auth
            
            def token
                cookies['monoster']
            end

            def check_auth
                if token == 1
                    json(404, 'error')
                end
            end
          end
          ",
            "
          module ConResponse 
            extend ActiveSupport::Concern
          
          def json(status, message, data = {})
            json={}
            json[:message] = message
            json[:data] = data unless data.empty?
            render :status => status, :json => json
          end

          end
          ",
        ];
        let expected = RubyFile {
            controllers: vec![Controller {
                name: "VersionController".to_string(),
                parent: "ApplicationController".to_string(),
                actions: Vec::new(),
                methods: vec![
                    Method{
                        name: "version".to_string(),
                        params: vec!["cat".to_string()],
                        returns: Vec::new(),
                        private: false
                    }
                ],
            }],
            modules: Vec::new(),
            requires: Vec::new(),
        };
        let actual = helper(main, files);
        assert_eq!(actual.is_ok(), true);
        assert_ne!(actual.unwrap(), expected, "don't know what the return object should be precisely but they should not be equal");
    }

    #[cfg(test)]
    mod helpers_and_concerns {
        #[ignore = "WIP"]
        #[test]
        fn basic() {}
    }

    #[cfg(test)]
    mod inheritance_class {
        #[ignore = "WIP"]
        #[test]
        fn basic() {}
    }
}
