// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![warn(clippy::all)]
//! Test SQL syntax specific to PostgreSQL. The parser based on the
//! generic dialect is also tested (on the inputs it can handle).

use sqlparser::ast::*;
use sqlparser::dialect::{GenericDialect, PostgreSqlDialect};
use sqlparser::parser::ParserError;
use sqlparser::test_utils::*;

#[test]
fn parse_create_table_with_defaults() {
    let sql = "CREATE TABLE public.customer (
            customer_id integer DEFAULT nextval(public.customer_customer_id_seq),
            store_id smallint NOT NULL,
            first_name character varying(45) NOT NULL,
            last_name character varying(45) COLLATE \"es_ES\" NOT NULL,
            email character varying(50),
            address_id smallint NOT NULL,
            activebool boolean DEFAULT true NOT NULL,
            create_date date DEFAULT now()::text NOT NULL,
            last_update timestamp without time zone DEFAULT now() NOT NULL,
            active integer NOT NULL
    ) WITH (fillfactor = 20, user_catalog_table = true, autovacuum_vacuum_threshold = 100)";
    match pg_and_generic().one_statement_parses_to(sql, "") {
        Statement::CreateTable {
            name,
            columns,
            constraints,
            with_options,
            if_not_exists: false,
            external: false,
            file_format: None,
            location: None,
            ..
        } => {
            assert_eq!("public.customer", name.to_string());
            assert_eq!(
                columns,
                vec![
                    ColumnDef {
                        name: "customer_id".into(),
                        data_type: DataType::Int,
                        collation: None,
                        options: vec![ColumnOptionDef {
                            name: None,
                            option: ColumnOption::Default(
                                pg().verified_expr("nextval(public.customer_customer_id_seq)")
                            )
                        }],
                    },
                    ColumnDef {
                        name: "store_id".into(),
                        data_type: DataType::SmallInt,
                        collation: None,
                        options: vec![ColumnOptionDef {
                            name: None,
                            option: ColumnOption::NotNull,
                        }],
                    },
                    ColumnDef {
                        name: "first_name".into(),
                        data_type: DataType::Varchar(Some(45)),
                        collation: None,
                        options: vec![ColumnOptionDef {
                            name: None,
                            option: ColumnOption::NotNull,
                        }],
                    },
                    ColumnDef {
                        name: "last_name".into(),
                        data_type: DataType::Varchar(Some(45)),
                        collation: Some(ObjectName(vec![Ident::with_quote('"', "es_ES")])),
                        options: vec![ColumnOptionDef {
                            name: None,
                            option: ColumnOption::NotNull,
                        }],
                    },
                    ColumnDef {
                        name: "email".into(),
                        data_type: DataType::Varchar(Some(50)),
                        collation: None,
                        options: vec![],
                    },
                    ColumnDef {
                        name: "address_id".into(),
                        data_type: DataType::SmallInt,
                        collation: None,
                        options: vec![ColumnOptionDef {
                            name: None,
                            option: ColumnOption::NotNull
                        }],
                    },
                    ColumnDef {
                        name: "activebool".into(),
                        data_type: DataType::Boolean,
                        collation: None,
                        options: vec![
                            ColumnOptionDef {
                                name: None,
                                option: ColumnOption::Default(Expr::Value(Value::Boolean(true))),
                            },
                            ColumnOptionDef {
                                name: None,
                                option: ColumnOption::NotNull,
                            }
                        ],
                    },
                    ColumnDef {
                        name: "create_date".into(),
                        data_type: DataType::Date,
                        collation: None,
                        options: vec![
                            ColumnOptionDef {
                                name: None,
                                option: ColumnOption::Default(
                                    pg().verified_expr("CAST(now() AS TEXT)")
                                )
                            },
                            ColumnOptionDef {
                                name: None,
                                option: ColumnOption::NotNull,
                            }
                        ],
                    },
                    ColumnDef {
                        name: "last_update".into(),
                        data_type: DataType::Timestamp,
                        collation: None,
                        options: vec![
                            ColumnOptionDef {
                                name: None,
                                option: ColumnOption::Default(pg().verified_expr("now()")),
                            },
                            ColumnOptionDef {
                                name: None,
                                option: ColumnOption::NotNull,
                            }
                        ],
                    },
                    ColumnDef {
                        name: "active".into(),
                        data_type: DataType::Int,
                        collation: None,
                        options: vec![ColumnOptionDef {
                            name: None,
                            option: ColumnOption::NotNull
                        }],
                    },
                ]
            );
            assert!(constraints.is_empty());
            assert_eq!(
                with_options,
                vec![
                    SqlOption {
                        name: "fillfactor".into(),
                        value: number("20")
                    },
                    SqlOption {
                        name: "user_catalog_table".into(),
                        value: Value::Boolean(true)
                    },
                    SqlOption {
                        name: "autovacuum_vacuum_threshold".into(),
                        value: number("100")
                    },
                ]
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_create_table_from_pg_dump() {
    let sql = "CREATE TABLE public.customer (
            customer_id integer DEFAULT nextval('public.customer_customer_id_seq'::regclass) NOT NULL,
            store_id smallint NOT NULL,
            first_name character varying(45) NOT NULL,
            last_name character varying(45) NOT NULL,
            info text[],
            address_id smallint NOT NULL,
            activebool boolean DEFAULT true NOT NULL,
            create_date date DEFAULT now()::DATE NOT NULL,
            create_date1 date DEFAULT 'now'::TEXT::date NOT NULL,
            last_update timestamp without time zone DEFAULT now(),
            release_year public.year,
            active integer
        )";
    pg().one_statement_parses_to(sql, "CREATE TABLE public.customer (\
            customer_id INT DEFAULT nextval(CAST('public.customer_customer_id_seq' AS REGCLASS)) NOT NULL, \
            store_id SMALLINT NOT NULL, \
            first_name CHARACTER VARYING(45) NOT NULL, \
            last_name CHARACTER VARYING(45) NOT NULL, \
            info TEXT[], \
            address_id SMALLINT NOT NULL, \
            activebool BOOLEAN DEFAULT true NOT NULL, \
            create_date DATE DEFAULT CAST(now() AS DATE) NOT NULL, \
            create_date1 DATE DEFAULT CAST(CAST('now' AS TEXT) AS DATE) NOT NULL, \
            last_update TIMESTAMP DEFAULT now(), \
            release_year public.year, \
            active INT\
        )");
}

#[test]
fn parse_create_table_with_inherit() {
    let sql = "\
               CREATE TABLE bazaar.settings (\
               settings_id UUID PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL, \
               user_id UUID UNIQUE, \
               value TEXT[], \
               use_metric BOOLEAN DEFAULT true\
               )";
    pg().verified_stmt(sql);
}

#[test]
fn parse_create_table_empty() {
    // Zero-column tables are weird, but supported by at least PostgreSQL.
    // <https://github.com/ballista-compute/sqlparser-rs/pull/94>
    let _ = pg_and_generic().verified_stmt("CREATE TABLE t ()");
}

#[test]
fn parse_create_table_constraints_only() {
    // Zero-column tables can also have constraints in PostgreSQL
    let sql = "CREATE TABLE t (CONSTRAINT positive CHECK (2 > 1))";
    let ast = pg_and_generic().verified_stmt(sql);
    match ast {
        Statement::CreateTable {
            name,
            columns,
            constraints,
            ..
        } => {
            assert_eq!("t", name.to_string());
            assert!(columns.is_empty());
            assert_eq!(
                only(constraints).to_string(),
                "CONSTRAINT positive CHECK (2 > 1)"
            );
        }
        _ => unreachable!(),
    };
}

#[test]
fn parse_create_table_if_not_exists() {
    let sql = "CREATE TABLE IF NOT EXISTS uk_cities ()";
    let ast = pg_and_generic().verified_stmt(sql);
    match ast {
        Statement::CreateTable {
            name,
            if_not_exists: true,
            ..
        } => {
            assert_eq!("uk_cities", name.to_string());
        }
        _ => unreachable!(),
    }
}

#[test]
fn parse_bad_if_not_exists() {
    let res = pg().parse_sql_statements("CREATE TABLE NOT EXISTS uk_cities ()");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: EXISTS".to_string()),
        res.unwrap_err()
    );

    let res = pg().parse_sql_statements("CREATE TABLE IF EXISTS uk_cities ()");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: EXISTS".to_string()),
        res.unwrap_err()
    );

    let res = pg().parse_sql_statements("CREATE TABLE IF uk_cities ()");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: uk_cities".to_string()),
        res.unwrap_err()
    );

    let res = pg().parse_sql_statements("CREATE TABLE IF NOT uk_cities ()");
    assert_eq!(
        ParserError::ParserError("Expected end of statement, found: NOT".to_string()),
        res.unwrap_err()
    );
}

#[test]
fn parse_copy_example() {
    let sql = r#"COPY public.actor (actor_id, first_name, last_name, last_update, value) FROM stdin;
1	PENELOPE	GUINESS	2006-02-15 09:34:33 0.11111
2	NICK	WAHLBERG	2006-02-15 09:34:33 0.22222
3	ED	CHASE	2006-02-15 09:34:33 0.312323
4	JENNIFER	DAVIS	2006-02-15 09:34:33 0.3232
5	JOHNNY	LOLLOBRIGIDA	2006-02-15 09:34:33 1.343
6	BETTE	NICHOLSON	2006-02-15 09:34:33 5.0
7	GRACE	MOSTEL	2006-02-15 09:34:33 6.0
8	MATTHEW	JOHANSSON	2006-02-15 09:34:33 7.0
9	JOE	SWANK	2006-02-15 09:34:33 8.0
10	CHRISTIAN	GABLE	2006-02-15 09:34:33 9.1
11	ZERO	CAGE	2006-02-15 09:34:33 10.001
12	KARL	BERRY	2017-11-02 19:15:42.308637+08 11.001
A Fateful Reflection of a Waitress And a Boat who must Discover a Sumo Wrestler in Ancient China
Kwara & Kogi
{"Deleted Scenes","Behind the Scenes"}
'awe':5 'awe-inspir':4 'barbarella':1 'cat':13 'conquer':16 'dog':18 'feminist':10 'inspir':6 'monasteri':21 'must':15 'stori':7 'streetcar':2
PHP	₱ USD $
\N  Some other value
\\."#;
    let ast = pg_and_generic().one_statement_parses_to(sql, "");
    println!("{:#?}", ast);
    //assert_eq!(sql, ast.to_string());
}

#[test]
fn parse_set() {
    let stmt = pg_and_generic().verified_stmt("SET a = b");
    assert_eq!(
        stmt,
        Statement::SetVariable {
            local: false,
            hivevar: false,
            variable: "a".into(),
            value: vec![SetVariableValue::Ident("b".into())],
        }
    );

    let stmt = pg_and_generic().verified_stmt("SET a = 'b'");
    assert_eq!(
        stmt,
        Statement::SetVariable {
            local: false,
            hivevar: false,
            variable: "a".into(),
            value: vec![SetVariableValue::Literal(Value::SingleQuotedString(
                "b".into()
            ))],
        }
    );

    let stmt = pg_and_generic().verified_stmt("SET a = 0");
    assert_eq!(
        stmt,
        Statement::SetVariable {
            local: false,
            hivevar: false,
            variable: "a".into(),
            value: vec![SetVariableValue::Literal(number("0"))],
        }
    );

    let stmt = pg_and_generic().verified_stmt("SET a = DEFAULT");
    assert_eq!(
        stmt,
        Statement::SetVariable {
            local: false,
            hivevar: false,
            variable: "a".into(),
            value: vec![SetVariableValue::Ident("DEFAULT".into())],
        }
    );

    let stmt = pg_and_generic().verified_stmt("SET LOCAL a = b");
    assert_eq!(
        stmt,
        Statement::SetVariable {
            local: true,
            hivevar: false,
            variable: "a".into(),
            value: vec![SetVariableValue::Ident("b".into())],
        }
    );

    pg_and_generic().one_statement_parses_to("SET a TO b", "SET a = b");
    pg_and_generic().one_statement_parses_to("SET SESSION a = b", "SET a = b");

    assert_eq!(
        pg_and_generic().parse_sql_statements("SET"),
        Err(ParserError::ParserError(
            "Expected identifier, found: EOF".to_string()
        )),
    );

    assert_eq!(
        pg_and_generic().parse_sql_statements("SET a b"),
        Err(ParserError::ParserError(
            "Expected equals sign or TO, found: b".to_string()
        )),
    );

    assert_eq!(
        pg_and_generic().parse_sql_statements("SET a ="),
        Err(ParserError::ParserError(
            "Expected variable value, found: EOF".to_string()
        )),
    );
}

#[test]
fn parse_show() {
    let stmt = pg_and_generic().verified_stmt("SHOW a");
    assert_eq!(
        stmt,
        Statement::ShowVariable {
            variable: "a".into()
        }
    );

    let stmt = pg_and_generic().verified_stmt("SHOW ALL");
    assert_eq!(
        stmt,
        Statement::ShowVariable {
            variable: "ALL".into()
        }
    )
}

fn pg() -> TestedDialects {
    TestedDialects {
        dialects: vec![Box::new(PostgreSqlDialect {})],
    }
}

fn pg_and_generic() -> TestedDialects {
    TestedDialects {
        dialects: vec![Box::new(PostgreSqlDialect {}), Box::new(GenericDialect {})],
    }
}
