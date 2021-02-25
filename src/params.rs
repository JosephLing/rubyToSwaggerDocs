use std::collections::{HashSet, VecDeque};

use lib_ruby_parser::Node;

fn search_for_param_in_list(statements: Vec<Node>, buf: &mut VecDeque<Box<Node>>) {
    for stat in statements {
        buf.push_back(Box::new((stat).clone()));
    }
}

fn optional_thing(body: &Option<Box<Node>>, buf: &mut VecDeque<Box<Node>>) {
    if let Some(body) = body {
        buf.push_back((*body).clone());
    }
}

// doesn't support inline methods and singleton classes
// search for: param, payload, headers
pub fn search_for_param(statement: Box<Node>) -> HashSet<String> {
    let mut params = HashSet::new();
    let mut buf = VecDeque::new();
    buf.push_back(statement);

    while !buf.is_empty() {
        match *buf.pop_front().unwrap() {
            Node::Alias(stat) => buf.push_back(stat.from),

            Node::And(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }
            Node::AndAsgn(stat) => buf.push_back(stat.value),

            Node::Array(stat) => search_for_param_in_list(stat.elements, &mut buf),
            Node::ArrayPattern(stat) => search_for_param_in_list(stat.elements, &mut buf),
            Node::ArrayPatternWithTail(stat) => search_for_param_in_list(stat.elements, &mut buf),

            Node::Begin(stat) => search_for_param_in_list(stat.statements, &mut buf),

            // note: ignore optional elements of block here
            Node::Block(stat) => optional_thing(&stat.body, &mut buf),
            Node::BlockPass(stat) => buf.push_back(stat.value),

            // Node::Case(stat) => {}
            // Node::CaseMatch(stat) => {}
            // Node::Casgn(stat) => {}
            // Node::Cbase(stat) => {}
            Node::Class(stat) => {
                if let Some(body) = stat.body {
                    buf.push_back(body);
                }
            }

            //TODO: do we need this???
            Node::Const(stat) => optional_thing(&stat.scope, &mut buf),

            Node::ConstPattern(stat) => buf.push_back(stat.pattern),

            // method name look up here!
            Node::CSend(stat) => search_for_param_in_list(stat.args, &mut buf),

            // accessing class stuff not needed
            // Node::Cvar(stat) => {}
            // Node::Cvasgn(stat) => {}
            Node::Defined(stat) => buf.push_back(stat.value),

            Node::Dstr(stat) => search_for_param_in_list(stat.parts, &mut buf),
            Node::Dsym(stat) => search_for_param_in_list(stat.parts, &mut buf),

            Node::EFlipFlop(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf)
            }

            Node::Ensure(stat) => {
                optional_thing(&stat.ensure, &mut buf);
                optional_thing(&stat.body, &mut buf)
            }

            Node::Erange(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf)
            }

            Node::FindPattern(stat) => search_for_param_in_list(stat.elements, &mut buf),

            Node::For(stat) => {
                buf.push_back(stat.iterator);
                buf.push_back(stat.iteratee);
                optional_thing(&stat.body, &mut buf);
            }

            // global vars
            // Node::Gvar(stat) => {}
            // Node::Gvasgn(stat) => {}
            Node::Hash(stat) => search_for_param_in_list(stat.pairs, &mut buf),
            Node::HashPattern(stat) => search_for_param_in_list(stat.elements, &mut buf),

            Node::If(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.if_true, &mut buf);
                optional_thing(&stat.if_false, &mut buf)
            }
            Node::IfGuard(stat) => buf.push_back(stat.cond),
            Node::IFlipFlop(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf);
            }
            Node::IfMod(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.if_true, &mut buf);
                optional_thing(&stat.if_false, &mut buf)
            }
            Node::IfTernary(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.if_true);
                buf.push_back(stat.if_false);
            }

            // special case!!!
            Node::Index(stat) => {
                // recv is params
                // index

                match *stat.recv {
                    Node::Const(con) => {
                        if con.name == "params" {
                            for index in stat.indexes {
                                match index {
                                    Node::Sym(value) => {
                                        params.insert(value.name.to_string_lossy());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Node::Send(send) => {
                        if send.method_name == "params" {
                            for index in stat.indexes {
                                match index {
                                    Node::Sym(value) => {
                                        params.insert(value.name.to_string_lossy());
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            search_for_param_in_list(send.args, &mut buf);
                            optional_thing(&send.recv, &mut buf);
                        }
                    }
                    _ => buf.push_back(stat.recv),
                }
            }

            // Node::IndexAsgn(stat) => {}
            Node::InPattern(stat) => {
                buf.push_back(stat.pattern);
                optional_thing(&stat.guard, &mut buf);
                optional_thing(&stat.body, &mut buf)
            }

            Node::Irange(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf);
            }

            // specail case!!!
            // Node::Ivar(stat) => stat.name,
            Node::Ivasgn(stat) => optional_thing(&stat.value, &mut buf),

            // Node::Kwarg(stat) => {}
            Node::Kwargs(stat) => search_for_param_in_list(stat.pairs, &mut buf),
            Node::KwBegin(stat) => search_for_param_in_list(stat.statements, &mut buf),
            Node::Kwoptarg(stat) => buf.push_back(stat.default),
            Node::Kwsplat(stat) => buf.push_back(stat.value),

            // Node::Lambda(stat) => {}

            // Node::Line(stat) => {}

            // Node::Lvar(stat) => {}

            // specail case for headers and payload!!!!
            Node::Lvasgn(stat) => optional_thing(&stat.value, &mut buf),

            Node::Masgn(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }

            Node::MatchAlt(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }
            Node::MatchAs(stat) => buf.push_back(stat.value),
            Node::MatchPattern(stat) => {
                buf.push_back(stat.value);
                buf.push_back(stat.pattern)
            }
            Node::MatchPatternP(stat) => {
                buf.push_back(stat.value);
                buf.push_back(stat.pattern)
            }
            Node::MatchRest(stat) => optional_thing(&stat.name, &mut buf),
            Node::MatchWithLvasgn(stat) => {
                buf.push_back(stat.re);
                buf.push_back(stat.value)
            }

            Node::Mlhs(stat) => search_for_param_in_list(stat.items, &mut buf),

            Node::Next(stat) => search_for_param_in_list(stat.args, &mut buf),

            Node::Numblock(stat) => buf.push_back(stat.body),

            Node::OpAsgn(stat) => {
                buf.push_back(stat.recv);
                buf.push_back(stat.value)
            }
            Node::Optarg(stat) => buf.push_back(stat.default),

            Node::Or(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }
            Node::OrAsgn(stat) => {
                buf.push_back(stat.recv);
                buf.push_back(stat.value)
            }

            Node::Pair(stat) => {
                buf.push_back(stat.key);
                buf.push_back(stat.value)
            }

            Node::Pin(stat) => buf.push_back(stat.var),

            Node::Postexe(stat) => optional_thing(&stat.body, &mut buf),
            Node::Preexe(stat) => optional_thing(&stat.body, &mut buf),
            Node::Procarg0(stat) => search_for_param_in_list(stat.args, &mut buf),

            Node::Regexp(stat) => {
                search_for_param_in_list(stat.parts, &mut buf);
                optional_thing(&stat.options, &mut buf)
            }

            Node::Rescue(stat) => {
                search_for_param_in_list(stat.rescue_bodies, &mut buf);
                optional_thing(&stat.else_, &mut buf);
                optional_thing(&stat.else_, &mut buf)
            }
            Node::RescueBody(stat) => {
                optional_thing(&stat.body, &mut buf);
                optional_thing(&stat.exc_var, &mut buf);
                optional_thing(&stat.exc_list, &mut buf)
            }

            Node::Return(stat) => search_for_param_in_list(stat.args, &mut buf),

            // TODO: important??
            // method name look up here!
            Node::Send(stat) => {
                if let Some(recv) = stat.recv.clone() {
                    if let Node::Send(send_param) = *recv {
                        if send_param.method_name == "params" {
                            for arg in stat.args {
                                match arg {
                                    Node::Sym(value) => {
                                        params.insert(value.name.to_string_lossy());
                                    }
                                    _ => {}
                                }
                            }
                        } else {
                            search_for_param_in_list(stat.args, &mut buf);
                            optional_thing(&stat.recv, &mut buf)
                        }
                    } else {
                        search_for_param_in_list(stat.args, &mut buf);
                        optional_thing(&stat.recv, &mut buf)
                    }
                } else {
                    search_for_param_in_list(stat.args, &mut buf);
                    optional_thing(&stat.recv, &mut buf)
                }
            }

            Node::Splat(stat) => optional_thing(&stat.value, &mut buf),

            Node::Undef(stat) => search_for_param_in_list(stat.names, &mut buf),
            Node::UnlessGuard(stat) => buf.push_back(stat.cond),
            Node::Until(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.body, &mut buf);
            }
            Node::UntilPost(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.body);
            }

            Node::When(stat) => {
                search_for_param_in_list(stat.patterns, &mut buf);
                optional_thing(&stat.body, &mut buf)
            }

            Node::While(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.body, &mut buf)
            }
            Node::WhilePost(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.body)
            }

            Node::Yield(stat) => search_for_param_in_list(stat.args, &mut buf),

            _ => {}
        }
    }

    params
}

#[cfg(test)]
mod params_tests {

    use lib_ruby_parser::Parser;
    use pretty_assertions::assert_eq;

    use super::search_for_param;

    fn helper(input: &str) -> String {
        let mut results = search_for_param(Box::new(
            Parser::new(input.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
        ))
        .into_iter()
        .collect::<Vec<String>>();
        results.sort();
        return results.join(", ");
    }

    #[test]
    fn send_method() {
        assert_eq!(helper("render 'show'"), "");
    }

    #[test]
    fn params_without_any_index() {
        assert_eq!(helper("params"), "");
    }

    #[test]
    fn params_index() {
        assert_eq!(helper("params[:id]"), "id");
    }

    #[test]
    fn params_require() {
        assert_eq!(
            helper("event_type = params.require(:issue_event_type_name)"),
            "issue_event_type_name"
        );
    }

    #[test]
    fn params_send() {
        assert_eq!(
            helper("@results = query.foo(params[:issue_event_type_name])"),
            "issue_event_type_name"
        );
    }

    #[test]
    fn params_require_complex() {
        assert_eq!(
            helper(
                " create_details =  {
            :project_key => params.require(:project_key),
            :issue_type_id => params.require(:issue_type_id),
            :title_field_key => p[:title_field_key],
            :description_field_key => p[:description_field_key],
            :title => p[:title],
            :description => p[:description]
          }"
            ),
            "issue_type_id, project_key"
        );
    }

    #[test]
    fn params_if() {
        assert_eq!(
            helper(
                "if params[:id]
                    @results = params[:cat]
                end"
            ),
            "cat, id"
        );
    }
}
