use std::{
    collections::{HashSet, VecDeque},
    iter::FromIterator,
};

use lib_ruby_parser::{nodes, Node};

use crate::params::search_for_param;

#[derive(Debug)]
enum AstError {
    NoName,
}

#[derive(Debug, PartialEq)]
pub struct Method {
    pub name: String,
    pub params: Vec<String>,
    pub returns: Vec<String>,
    pub private: bool,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.private {
            write!(f, "{} private", self.name)?;
        } else {
            write!(f, "{} ", self.name)?;

            if !self.params.is_empty() {
                write!(f, "params: {} ", self.params.join(","))?;
            }

            if !self.returns.is_empty() {
                write!(f, "responses: {} ", self.returns.join(","))?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Action {
    BeforeAction(String),
    AroundAction(String),
    SkipAuthMethods(String),
    SkipBeforeAction(String),
}

#[derive(Debug, PartialEq)]
pub struct Controller {
    pub name: String,
    pub parent: String,
    pub methods: Vec<Method>,
    pub actions: Vec<Action>,
}

#[derive(Debug, PartialEq)]
pub struct Module {
    pub name: String,
    pub classes: Vec<Controller>,
    pub requires: Vec<String>,
    pub methods: Vec<Method>,
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "module: '{}'", self.name)?;

        for i in 0..self.requires.len() {
            writeln!(f, "requires '{}'", self.requires[i])?;
        }

        for i in 0..self.classes.len() {
            writeln!(f, "{}", self.classes[i])?;
        }

        for i in 0..self.methods.len() {
            writeln!(f, "{}", self.methods[i])?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.parent.is_empty() {
            writeln!(f, "class {}", self.name)?;
        } else {
            writeln!(f, "class {} < {}", self.name, self.parent)?;
        }
        for i in 0..self.methods.len() {
            writeln!(f, "{}", self.methods[i])?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct RubyFile {
    pub controllers: Vec<Controller>,
    pub modules: Vec<Module>,
    pub requires: Vec<String>,
}

impl std::fmt::Display for RubyFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.requires.len() {
            writeln!(f, "requires '{}'", self.requires[i])?;
        }
        for i in 0..self.modules.len() {
            writeln!(f, "module '{}'", self.modules[i])?;
        }
        for i in 0..self.controllers.len() {
            writeln!(f, "{}", self.controllers[i])?;
        }
        Ok(())
    }
}

fn get_node_name(name: &Node) -> Result<String, AstError> {
    match name {
        Node::Const(node_const_name) => {
            if let Some(scope) = &node_const_name.scope {
                Ok(format!(
                    "{}::{}",
                    get_node_name(scope)?,
                    node_const_name.name
                ))
            } else {
                Ok(node_const_name.name.to_string())
            }
        }
        _ => Err(AstError::NoName),
    }
}

fn pretty_print(e: Node) -> String {
    match e {
        Node::Sym(sym) => sym.name.to_string_lossy(),
        Node::Ivar(ivar) => ivar.name,
        Node::Str(str) => str.value.to_string_lossy(),
        Node::Int(int) => int.value,
        _ => "unknown".to_string(),
    }
}

// a lot of this work is around how do we get the AST to represent the actual code...
// however in the future config_parse_list should give a Config object with all things in it in types....
fn config_parse(node: Node) -> String {
    let mut results = "".to_string();

    let mut buf = VecDeque::new();

    buf.push_back(node);

    while !buf.is_empty() {
        let temp = buf.pop_front().unwrap();
        results += &match temp {
            Node::Sym(sym) => sym.name.to_string_lossy(),
            Node::Ivar(ivar) => ivar.name.to_string(),
            Node::Str(str) => str.value.to_string_lossy().clone(),
            Node::Int(int) => int.value.to_string(),
            Node::Pair(pair) => {
                buf.push_back(*pair.key);
                buf.push_back(*pair.value);
                ":".to_string()
            }
            Node::Hash(hash) => {
                for pair in hash.pairs {
                    buf.push_back(pair);
                }
                " ".to_string()
            }
            Node::Kwargs(kwargs) => {
                for pair in kwargs.pairs {
                    buf.push_back(pair);
                }
                " ".to_string()
            }
            Node::Array(array) => {
                for elem in array.elements {
                    buf.push_back(elem);
                }
                "[array] ".to_string()
            }
            _ => {
                // println!("{:?}", temp);
                "".to_string()
            }
        };
    }

    // println!("result: {} {:?}", results, buf);
    results
}
//TODO: work out how to parse ruby config!
// dry language but everything is configured with all these bits of strange code all over the place
// good for a single file but it's hard to piece together everything specailly without nice tooling (atm - from what I have found)
fn config_parse_list(args: &Vec<Node>) -> String {
    let mut results = "".to_string();
    for arg in args {
        results += &config_parse((*arg).clone());
    }

    results
}

fn parse_def(def: &lib_ruby_parser::nodes::Def, private: bool) -> Result<Method, String> {
    let mut params = HashSet::new();
    let mut returns = Vec::new();
    if let Some(body) = def.body.clone() {
        match *body {
            Node::Begin(begin) => {
                for i in 0..begin.statements.len() {
                    match &begin.statements[i] {
                        // return statement - currently doesn't support another statement doing a return e.g. a if statement
                        Node::Send(send) => {
                            let recv = if let Some(recv) = &send.recv {
                                pretty_print((**recv).clone())
                            } else {
                                "none".to_string()
                            };
                            let args = send
                                .args
                                .clone()
                                .into_iter()
                                .map(|e| pretty_print(e))
                                .collect::<Vec<String>>();

                            let mut return_msg = send.method_name.clone();
                            if !recv.is_empty() {
                                return_msg += &format!(" {}", recv);
                            }
                            if !args.is_empty() {
                                return_msg += &format!(" {}", args.join(" "));
                            }

                            returns.push(return_msg);
                        }

                        // otherwise search for use of params!
                        _ => {
                            let foo = search_for_param(Box::new(begin.statements[i].clone()));
                            params.extend::<Vec<String>>(foo.into_iter().collect::<Vec<String>>());
                        }
                    }
                }
            }
            Node::OrAsgn(or_asign) => {
                // orAsign.recv -- just need the name maybe... or could just ignore
                match *or_asign.recv {
                    Node::Ivasgn(ivasgn) => {
                        println!("or assign: {}", ivasgn.name);
                        search_for_param(or_asign.value);
                    }
                    _ => println!("error unknown node for orAsign"),
                }
                // orAsign.value -- run get params on this
            }
            _ => {}
        }
    }

    Ok(Method {
        name: def.name.clone(),
        params: Vec::from_iter(params),
        returns,
        private,
    })
}

fn parse_class(class: nodes::Class) -> Result<Controller, String> {
    let name = get_node_name(&class.name).unwrap();
    let parent = get_node_name(&class.superclass.unwrap()).unwrap();

    let mut methods = Vec::new();
    let mut actions = Vec::new();

    if parent == "ActionController::API" {
        println!("oooh boy {} {}", name, parent);
    }

    if let Some(body) = class.body {
        match *body {
            Node::Begin(begin) => {
                let mut private = false;
                for i in 0..begin.statements.len() {
                    match &begin.statements[i] {
                        Node::Def(def) => {
                            if let Ok(method) = parse_def(&def, private) {
                                methods.push(method);
                            }
                        }
                        Node::Send(send) => {
                            let action_name = config_parse_list(&send.args);

                            match send.method_name.as_str() {
                                "private" => {
                                    private = true;
                                }
                                "skip_auth_methods" => {
                                    actions.push(Action::SkipAuthMethods(action_name));
                                }
                                "around_action" => {
                                    actions.push(Action::AroundAction(action_name));
                                }
                                "before_action" => {
                                    actions.push(Action::BeforeAction(action_name));
                                }
                                "skip_before_action" => {
                                    // println!("{:?}", send);
                                    actions.push(Action::SkipBeforeAction(action_name));
                                }
                                _ => {}
                            }
                        }
                        _ => println!("error unknown statement found in class"),
                    }
                }

                Ok(Controller {
                    name,
                    parent,
                    methods: methods,
                    actions,
                })
            }
            Node::Def(def) => {
                if let Ok(method) = parse_def(&def, false) {
                    methods.push(method);
                }

                Ok(Controller {
                    name,
                    parent,
                    methods: methods,
                    actions,
                })
            }
            _ => Err("no class body found".to_string()),
        }
    } else {
        Err("no class body found".to_string())
    }
}

fn parse_module(module: nodes::Module, parent_name: &str) -> Result<Vec<Module>, String> {
    let module_name = if parent_name.is_empty() {
        get_node_name(&module.name).unwrap()
    } else {
        format!("{}.{}", parent_name, get_node_name(&module.name).unwrap())
    };
    let mut requires = Vec::new();
    let mut classes = Vec::new();
    let mut modules = Vec::new();
    let mut methods = Vec::new();
    let mut private = false;
    if let Some(body) = module.body {
        match *body {
            Node::Class(class) => classes.push(parse_class(class)?),
            Node::Begin(begin) => {
                for i in 0..begin.statements.len() {
                    match begin.statements[i].clone() {
                        Node::Module(module) => {
                            modules.append(&mut parse_module(module, &module_name)?);
                        }
                        Node::Class(class) => classes.push(parse_class(class)?),
                        Node::Send(send) => {
                            if send.method_name == "require" {
                                for i in 0..send.args.len() {
                                    requires.push(pretty_print(send.args[i].clone()));
                                }
                            }
                            if send.method_name == "private" {
                                private = true;
                            }
                        }

                        // assignment outside the controller isn't supported
                        Node::Casgn(_) => {}

                        //NOTE: block is currently not supported, however doesn't look cruical atm until we do authentication handling for routes
                        Node::Block(_) => {}

                        Node::Def(def) => {
                            methods.push(parse_def(&def, private)?);
                        }
                        _ => {
                            println!("{:?}", begin.statements[i]);
                            Err("Unexpected node type found")?
                        }
                    };
                }
            }
            _ => {}
        }
    } else {
        Err("no module body found")?
    }

    modules.push(Module {
        name: module_name,
        classes,
        requires,
        methods,
    });

    Ok(modules)
}

pub fn parse_file(ast: Node) -> Result<RubyFile, String> {
    let mut requires = Vec::new();
    let mut classes = Vec::new();
    let mut modules = Vec::new();
    match ast {
        Node::Module(module) => {
            modules.append(&mut parse_module(module, "")?);
        }
        Node::Class(class) => classes.push(parse_class(class)?),
        Node::Begin(begin) => {
            for i in 0..begin.statements.len() {
                match begin.statements[i].clone() {
                    Node::Module(module) => {
                        modules.append(&mut parse_module(module, "")?);
                    }
                    Node::Class(class) => classes.push(parse_class(class)?),
                    Node::Send(send) => {
                        if send.method_name == "require" {
                            for i in 0..send.args.len() {
                                requires.push(pretty_print(send.args[i].clone()));
                            }
                        }
                    }
                    _ => Err("error no module found")?,
                };
            }
        }
        _ => Err("error no module found")?,
    }
    if classes.is_empty() && modules.is_empty() {
        Err("no classes and modules found - as empty")?
    }
    Ok(RubyFile {
        controllers: classes,
        modules: modules,
        requires,
    })
}

#[cfg(test)]
mod file_tests {

    use crate::ruby_parser::{parse_file, Action, Controller, Method, Module, RubyFile};
    use lib_ruby_parser::{Node, Parser};
    use pretty_assertions::assert_eq;
    use std::vec;

    fn helper(input: &str) -> Node {
        return Parser::new(input.as_bytes(), Default::default())
            .do_parse()
            .ast
            .unwrap();
    }

    #[test]
    fn parse_test() {
        let input = "class VersionController < ApplicationController

        skip_before_action :check_auth_token, only: [:version]
      
        def version
          not_found and return if ['eu-prod', 'us-prod'].include? ENV['GKE_CLUSTER']
      
          render json: {
            commit: ENV['COMMIT'],
            deployer: ENV['DEPLOYER'],
            deployed_at: ENV['DEPLOYED_AT']
          }
        end
      end";

        let expected = RubyFile {
            controllers: vec![Controller {
                name: "VersionController".to_string(),
                parent: "ApplicationController".to_string(),
                actions: vec![Action::SkipBeforeAction(
                    "check_auth_token :only[array] version".to_string(),
                )],
                methods: vec![Method {
                    name: "version".to_string(),
                    params: Vec::new(),
                    returns: vec!["render none unknown".to_string()],
                    private: false,
                }],
            }],
            modules: Vec::new(),
            requires: Vec::new(),
        };
        let actual = parse_file(helper(input));
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn parse_test2() {
        let input = "
        require 'jwt'
        class VersionController < ApplicationController

        skip_before_action :check_auth_token
      
        def version
          not_found and return if ['eu-prod', 'us-prod'].include? ENV['GKE_CLUSTER']
      
          render json: {
            commit: ENV['COMMIT'],
            deployer: ENV['DEPLOYER'],
            deployed_at: ENV['DEPLOYED_AT']
          }
        end
      end";

        let expected = RubyFile {
            controllers: vec![Controller {
                name: "VersionController".to_string(),
                parent: "ApplicationController".to_string(),
                actions: vec![Action::SkipBeforeAction("check_auth_token".to_string())],
                methods: vec![Method {
                    name: "version".to_string(),
                    params: Vec::new(),
                    returns: vec!["render none unknown".to_string()],
                    private: false,
                }],
            }],
            modules: Vec::new(),
            requires: vec!["jwt".to_string()],
        };
        let actual = parse_file(helper(input));
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn parse_test3() {
        let input = "
        module Widget
            Setting = Struct.new(:name, :value)
            class IntegrationSettingsController < ApplicationController
                include Widget::SnippetParser

                skip_auth_methods

                def index
                begin
                    snippet = parse_snippet(params, true, false)
                rescue JSON::ParserError => e
                    return json(400, 'malformed JSON #{e}')
                end
                end
            end
            end
        ";
        let expected = RubyFile {
            controllers: Vec::new(),
            modules: vec![Module {
                name: "Widget".to_string(),
                methods: Vec::new(),
                classes: vec![Controller {
                    actions: vec![Action::SkipAuthMethods("".to_string())],
                    name: "IntegrationSettingsController".to_string(),
                    parent: "ApplicationController".to_string(),
                    methods: vec![Method {
                        name: "index".to_string(),
                        params: Vec::new(),
                        returns: Vec::new(),
                        private: false,
                    }],
                }],
                requires: Vec::new(),
            }],
            requires: Vec::new(),
        };
        let actual = parse_file(helper(input));
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn parse_test4() {
        let input = "
        module Widget
            Setting = Struct.new(:name, :value)
            class Vendors::IntegrationSettingsController < ApplicationController
                include Widget::SnippetParser

                skip_auth_methods

                def index
                begin
                    snippet = parse_snippet(params, true, false)
                rescue JSON::ParserError => e
                    return json(400, 'malformed JSON #{e}')
                end
                end
            end
            end
        ";

        let expected = RubyFile {
            controllers: Vec::new(),
            modules: vec![Module {
                name: "Widget".to_string(),
                methods: Vec::new(),
                classes: vec![Controller {
                    name: "Vendors::IntegrationSettingsController".to_string(),
                    parent: "ApplicationController".to_string(),
                    actions: vec![Action::SkipAuthMethods("".to_string())],
                    methods: vec![Method {
                        name: "index".to_string(),
                        params: Vec::new(),
                        returns: Vec::new(),
                        private: false,
                    }],
                }],
                requires: Vec::new(),
            }],
            requires: Vec::new(),
        };
        let actual = parse_file(helper(input));
        println!("{:?}", actual);
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn parse_test5() {
        let input = "
        module SortParams
            extend ActiveSupport::Concern

            def sorted_fields(sort, allowed, default)
                foo
            end
        end
        ";

        let expected = RubyFile {
            controllers: Vec::new(),
            modules: vec![Module {
                name: "SortParams".to_string(),
                methods: vec![Method {
                    name: "sorted_fields".to_string(),
                    params: Vec::new(),
                    returns: Vec::new(),
                    private: false,
                }],
                classes: Vec::new(),
                requires: Vec::new(),
            }],
            requires: Vec::new(),
        };
        let actual = parse_file(helper(input));
        println!("{:?}", actual);
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap(), expected);
    }


    #[test]
    fn parse_test6() {
        let input = "
        class VersionController < ApplicationController

            def version
                json(200, 'version', param[:cat])
            end
        end
        ";

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
        let actual = parse_file(helper(input));
        println!("{:?}", actual);
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap(), expected);
    }
}
