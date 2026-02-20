//! Upstream `examples.ts` template catalog.
//!
//! Rust divergence:
//! - Upstream exports many `const` template values; this module exposes
//!   constructor functions returning `Template` values.
//! - Upstream `Date.now()` calls are mapped to `now_millis()` at generation
//!   time.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::string::Token;
use crate::structured::{ObjectTemplateField, Template, TemplateJson};

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn req(key: &str, value: Template) -> ObjectTemplateField {
    ObjectTemplateField::required_literal_key(key, value)
}

fn opt(key: &str, value: Template, optionality: f64) -> ObjectTemplateField {
    ObjectTemplateField::optional_literal_key(key, value, optionality)
}

pub fn token_email() -> Token {
    Token::list(vec![
        Token::repeat(3, 12, Token::char_range(97, 122, None)),
        Token::pick(vec![
            Token::literal("."),
            Token::literal("_"),
            Token::literal("-"),
            Token::literal(""),
        ]),
        Token::repeat(0, 5, Token::char_range(97, 122, None)),
        Token::literal("@"),
        Token::pick(vec![
            Token::literal("gmail.com"),
            Token::literal("yahoo.com"),
            Token::literal("example.org"),
            Token::literal("test.com"),
            Token::literal("demo.net"),
        ]),
    ])
}

pub fn token_phone() -> Token {
    Token::list(vec![
        Token::literal("+1-"),
        Token::char_range(50, 57, Some(3)),
        Token::literal("-"),
        Token::char_range(48, 57, Some(3)),
        Token::literal("-"),
        Token::char_range(48, 57, Some(4)),
    ])
}

pub fn token_product_code() -> Token {
    Token::list(vec![
        Token::pick(vec![
            Token::literal("PRD"),
            Token::literal("ITM"),
            Token::literal("SKU"),
        ]),
        Token::literal("-"),
        Token::char_range(65, 90, Some(2)),
        Token::char_range(48, 57, Some(6)),
    ])
}

pub fn token_url() -> Token {
    Token::list(vec![
        Token::literal("https://"),
        Token::repeat(3, 15, Token::char_range(97, 122, None)),
        Token::pick(vec![
            Token::literal(".com"),
            Token::literal(".org"),
            Token::literal(".net"),
            Token::literal(".io"),
        ]),
        Token::pick(vec![
            Token::literal(""),
            Token::literal("/"),
            Token::literal("/api/"),
            Token::literal("/v1/"),
        ]),
        Token::repeat(0, 10, Token::char_range(97, 122, None)),
    ])
}

pub fn token_username() -> Token {
    Token::list(vec![
        Token::pick(vec![
            Token::literal("user"),
            Token::literal("admin"),
            Token::literal("guest"),
            Token::literal("test"),
        ]),
        Token::char_range(48, 57, Some(4)),
    ])
}

pub fn user_profile() -> Template {
    Template::obj(vec![
        req("id", Template::int(Some(1), Some(10_000))),
        req("username", Template::str(Some(token_username()))),
        req("email", Template::str(Some(token_email()))),
        req("age", Template::int(Some(18), Some(120))),
        req("isActive", Template::bool(None)),
        req(
            "profile",
            Template::obj(vec![
                req(
                    "bio",
                    Template::str(Some(Token::repeat(
                        10,
                        50,
                        Token::char_range(32, 126, None),
                    ))),
                ),
                opt(
                    "avatar",
                    Template::str(Some(Token::list(vec![
                        Token::literal("https://avatar.example.com/"),
                        Token::char_range(48, 57, Some(8)),
                    ]))),
                    0.3,
                ),
            ]),
        ),
    ])
}

pub fn user_basic() -> Template {
    Template::obj(vec![
        req("id", Template::int(Some(1), Some(1000))),
        req(
            "name",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("John"),
                    Token::literal("Jane"),
                    Token::literal("Bob"),
                    Token::literal("Alice"),
                    Token::literal("Charlie"),
                ]),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("Doe"),
                    Token::literal("Smith"),
                    Token::literal("Johnson"),
                    Token::literal("Brown"),
                ]),
            ]))),
        ),
        req("active", Template::bool(None)),
    ])
}

pub fn api_response() -> Template {
    Template::obj(vec![
        req(
            "status",
            Template::str(Some(Token::pick(vec![
                Token::literal("success"),
                Token::literal("error"),
            ]))),
        ),
        req(
            "timestamp",
            Template::int(Some(1_640_000_000), Some(1_700_000_000)),
        ),
        req(
            "data",
            Template::arr(
                Some(0),
                Some(10),
                Some(Template::obj(vec![
                    req("id", Template::int(None, None)),
                    req("value", Template::str(None)),
                ])),
                vec![],
                vec![],
            ),
        ),
    ])
}

pub fn api_response_detailed() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req("success", Template::lit(Value::Bool(true))),
        req("timestamp", Template::lit(Value::Number(now.into()))),
        req(
            "data",
            Template::arr(
                Some(5),
                Some(15),
                Some(Template::obj(vec![
                    req("id", Template::int(None, None)),
                    req(
                        "status",
                        Template::str(Some(Token::pick(vec![
                            Token::literal("pending"),
                            Token::literal("completed"),
                            Token::literal("failed"),
                        ]))),
                    ),
                    req("value", Template::float(Some(0.0), Some(1000.0))),
                ])),
                vec![],
                vec![],
            ),
        ),
    ])
}

pub fn service_config() -> Template {
    Template::obj(vec![
        req(
            "database",
            Template::obj(vec![
                req(
                    "host",
                    Template::str(Some(Token::list(vec![
                        Token::literal("db-"),
                        Token::char_range(48, 57, Some(2)),
                        Token::literal(".example.com"),
                    ]))),
                ),
                req("port", Template::int(Some(3000), Some(6000))),
                req("timeout", Template::int(Some(1000), Some(30000))),
                req("pool_size", Template::int(Some(5), Some(50))),
            ]),
        ),
        req(
            "cache",
            Template::obj(vec![
                req("enabled", Template::bool(None)),
                req("ttl", Template::int(Some(60), Some(3600))),
                req("max_size", Template::int(Some(100), Some(10_000))),
            ]),
        ),
        req(
            "features",
            Template::map(
                Some(Token::pick(vec![
                    Token::literal("feature_a"),
                    Token::literal("feature_b"),
                    Token::literal("feature_c"),
                    Token::literal("feature_d"),
                ])),
                Some(Template::bool(None)),
                Some(2),
                Some(5),
            ),
        ),
    ])
}

pub fn config_map() -> Template {
    Template::map(
        Some(Token::pick(vec![
            Token::literal("timeout"),
            Token::literal("retries"),
            Token::literal("cache_ttl"),
            Token::literal("max_connections"),
        ])),
        Some(Template::int(Some(1), Some(3600))),
        Some(3),
        Some(5),
    )
}

pub fn permissions() -> Template {
    Template::map(
        Some(Token::list(vec![
            Token::literal("can_"),
            Token::pick(vec![
                Token::literal("read"),
                Token::literal("write"),
                Token::literal("delete"),
                Token::literal("admin"),
            ]),
        ])),
        Some(Template::bool(None)),
        Some(2),
        Some(6),
    )
}

pub fn translations() -> Template {
    Template::map(
        Some(Token::pick(vec![
            Token::literal("welcome"),
            Token::literal("goodbye"),
            Token::literal("error"),
            Token::literal("success"),
            Token::literal("loading"),
        ])),
        Some(Template::str(Some(Token::repeat(
            5,
            20,
            Token::char_range(32, 126, None),
        )))),
        Some(3),
        Some(8),
    )
}

pub fn tree() -> Template {
    Template::obj(vec![
        req("value", Template::int(None, None)),
        opt("left", Template::recursive(tree), 0.3),
        opt("right", Template::recursive(tree), 0.3),
    ])
}

pub fn comment() -> Template {
    Template::obj(vec![
        req("id", Template::int(None, None)),
        req("text", Template::str(None)),
        req("author", Template::str(None)),
        opt(
            "replies",
            Template::arr(
                Some(0),
                Some(3),
                Some(Template::recursive(comment)),
                vec![],
                vec![],
            ),
            0.4,
        ),
    ])
}

pub fn product() -> Template {
    Template::obj(vec![
        req(
            "id",
            Template::str(Some(Token::list(vec![
                Token::literal("prod_"),
                Token::char_range(48, 57, Some(8)),
            ]))),
        ),
        req(
            "name",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("Premium"),
                    Token::literal("Deluxe"),
                    Token::literal("Classic"),
                    Token::literal("Modern"),
                    Token::literal("Vintage"),
                ]),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("Widget"),
                    Token::literal("Gadget"),
                    Token::literal("Tool"),
                    Token::literal("Device"),
                    Token::literal("Accessory"),
                ]),
            ]))),
        ),
        req("price", Template::float(Some(9.99), Some(999.99))),
        req(
            "currency",
            Template::str(Some(Token::pick(vec![
                Token::literal("USD"),
                Token::literal("EUR"),
                Token::literal("GBP"),
                Token::literal("JPY"),
            ]))),
        ),
        req(
            "category",
            Template::str(Some(Token::pick(vec![
                Token::literal("electronics"),
                Token::literal("clothing"),
                Token::literal("books"),
                Token::literal("home"),
                Token::literal("sports"),
            ]))),
        ),
        req(
            "tags",
            Template::arr(
                Some(1),
                Some(5),
                Some(Template::str(Some(Token::pick(vec![
                    Token::literal("new"),
                    Token::literal("sale"),
                    Token::literal("featured"),
                    Token::literal("popular"),
                    Token::literal("limited"),
                ])))),
                vec![],
                vec![],
            ),
        ),
        req(
            "inventory",
            Template::obj(vec![
                req("stock", Template::int(Some(0), Some(1000))),
                req(
                    "warehouse",
                    Template::str(Some(Token::list(vec![
                        Token::literal("WH-"),
                        Token::char_range(65, 90, Some(2)),
                        Token::char_range(48, 57, Some(3)),
                    ]))),
                ),
                req("reserved", Template::int(Some(0), Some(50))),
            ]),
        ),
        req("rating", Template::float(Some(1.0), Some(5.0))),
        req("reviews", Template::int(Some(0), Some(10_000))),
    ])
}

pub fn order() -> Template {
    Template::obj(vec![
        req(
            "orderId",
            Template::str(Some(Token::list(vec![
                Token::literal("ORD-"),
                Token::char_range(48, 57, Some(10)),
            ]))),
        ),
        req(
            "customerId",
            Template::str(Some(Token::list(vec![
                Token::literal("CUST-"),
                Token::char_range(65, 90, Some(3)),
                Token::char_range(48, 57, Some(6)),
            ]))),
        ),
        req(
            "items",
            Template::arr(
                Some(1),
                Some(8),
                Some(Template::obj(vec![
                    req("productId", Template::str(Some(token_product_code()))),
                    req("quantity", Template::int(Some(1), Some(10))),
                    req("price", Template::float(Some(5.0), Some(500.0))),
                ])),
                vec![],
                vec![],
            ),
        ),
        req("total", Template::float(Some(10.0), Some(2000.0))),
        req(
            "status",
            Template::str(Some(Token::pick(vec![
                Token::literal("pending"),
                Token::literal("processing"),
                Token::literal("shipped"),
                Token::literal("delivered"),
                Token::literal("cancelled"),
            ]))),
        ),
        req(
            "createdAt",
            Template::int(Some(1_640_000_000), Some(1_700_000_000)),
        ),
        req(
            "shippingAddress",
            Template::obj(vec![
                req(
                    "street",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(48, 57, Some(3)),
                        Token::literal(" "),
                        Token::pick(vec![
                            Token::literal("Main St"),
                            Token::literal("Oak Ave"),
                            Token::literal("Pine Rd"),
                            Token::literal("Cedar Ln"),
                        ]),
                    ]))),
                ),
                req(
                    "city",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("New York"),
                        Token::literal("Los Angeles"),
                        Token::literal("Chicago"),
                        Token::literal("Houston"),
                        Token::literal("Phoenix"),
                    ]))),
                ),
                req(
                    "state",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("NY"),
                        Token::literal("CA"),
                        Token::literal("IL"),
                        Token::literal("TX"),
                        Token::literal("AZ"),
                    ]))),
                ),
                req(
                    "zipCode",
                    Template::str(Some(Token::char_range(48, 57, Some(5)))),
                ),
                req(
                    "country",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("US"),
                        Token::literal("CA"),
                        Token::literal("MX"),
                    ]))),
                ),
            ]),
        ),
    ])
}

pub fn user_token() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "token",
            Template::str(Some(Token::list(vec![
                Token::literal("eyJ"),
                Token::repeat(20, 40, Token::char_range(65, 90, None)),
            ]))),
        ),
        req(
            "refreshToken",
            Template::str(Some(Token::list(vec![
                Token::literal("rt_"),
                Token::repeat(32, 64, Token::char_range(97, 122, None)),
            ]))),
        ),
        req(
            "expiresAt",
            Template::int(Some(now), Some(now.saturating_add(86_400_000))),
        ),
        req(
            "scope",
            Template::arr(
                Some(1),
                Some(4),
                Some(Template::str(Some(Token::pick(vec![
                    Token::literal("read"),
                    Token::literal("write"),
                    Token::literal("admin"),
                    Token::literal("user"),
                ])))),
                vec![],
                vec![],
            ),
        ),
    ])
}

pub fn user_role() -> Template {
    Template::obj(vec![
        req(
            "roleId",
            Template::str(Some(Token::list(vec![
                Token::literal("role_"),
                Token::char_range(48, 57, Some(6)),
            ]))),
        ),
        req(
            "name",
            Template::str(Some(Token::pick(vec![
                Token::literal("admin"),
                Token::literal("user"),
                Token::literal("moderator"),
                Token::literal("guest"),
                Token::literal("super_admin"),
            ]))),
        ),
        req(
            "permissions",
            Template::arr(
                Some(2),
                Some(10),
                Some(Template::str(Some(Token::pick(vec![
                    Token::literal("users:read"),
                    Token::literal("users:write"),
                    Token::literal("users:delete"),
                    Token::literal("posts:read"),
                    Token::literal("posts:write"),
                    Token::literal("posts:delete"),
                    Token::literal("admin:read"),
                    Token::literal("admin:write"),
                    Token::literal("system:manage"),
                ])))),
                vec![],
                vec![],
            ),
        ),
        req("createdBy", Template::str(Some(token_username()))),
        req("isActive", Template::bool(None)),
    ])
}

pub fn log_entry() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "timestamp",
            Template::int(Some(now.saturating_sub(86_400_000)), Some(now)),
        ),
        req(
            "level",
            Template::str(Some(Token::pick(vec![
                Token::literal("debug"),
                Token::literal("info"),
                Token::literal("warn"),
                Token::literal("error"),
                Token::literal("fatal"),
            ]))),
        ),
        req(
            "message",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("User"),
                    Token::literal("System"),
                    Token::literal("Database"),
                    Token::literal("API"),
                    Token::literal("Cache"),
                ]),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("action"),
                    Token::literal("error"),
                    Token::literal("warning"),
                    Token::literal("success"),
                    Token::literal("failure"),
                ]),
                Token::literal(": "),
                Token::repeat(10, 50, Token::char_range(32, 126, None)),
            ]))),
        ),
        req(
            "service",
            Template::str(Some(Token::pick(vec![
                Token::literal("web-server"),
                Token::literal("database"),
                Token::literal("cache"),
                Token::literal("auth-service"),
                Token::literal("api-gateway"),
            ]))),
        ),
        opt("userId", Template::str(Some(token_username())), 0.7),
        req(
            "requestId",
            Template::str(Some(Token::list(vec![
                Token::literal("req_"),
                Token::char_range(97, 122, Some(8)),
                Token::char_range(48, 57, Some(4)),
            ]))),
        ),
        opt(
            "metadata",
            Template::obj(vec![
                req(
                    "ip",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(49, 57, Some(1)),
                        Token::char_range(48, 57, Some(2)),
                        Token::literal("."),
                        Token::char_range(48, 57, Some(3)),
                        Token::literal("."),
                        Token::char_range(48, 57, Some(3)),
                        Token::literal("."),
                        Token::char_range(48, 57, Some(3)),
                    ]))),
                ),
                req(
                    "userAgent",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("Mozilla/5.0 (Windows NT 10.0; Win64; x64)"),
                        Token::literal("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"),
                        Token::literal("Mozilla/5.0 (X11; Linux x86_64)"),
                    ]))),
                ),
                req("duration", Template::int(Some(1), Some(5000))),
            ]),
            0.5,
        ),
    ])
}

pub fn metric_data() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "name",
            Template::str(Some(Token::pick(vec![
                Token::literal("cpu_usage"),
                Token::literal("memory_usage"),
                Token::literal("disk_io"),
                Token::literal("network_latency"),
                Token::literal("request_count"),
            ]))),
        ),
        req("value", Template::float(Some(0.0), Some(100.0))),
        req(
            "unit",
            Template::str(Some(Token::pick(vec![
                Token::literal("percent"),
                Token::literal("bytes"),
                Token::literal("ms"),
                Token::literal("count"),
                Token::literal("rate"),
            ]))),
        ),
        req(
            "timestamp",
            Template::int(Some(now.saturating_sub(3_600_000)), Some(now)),
        ),
        req(
            "tags",
            Template::obj(vec![
                req(
                    "environment",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("production"),
                        Token::literal("staging"),
                        Token::literal("development"),
                    ]))),
                ),
                req(
                    "service",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("web"),
                        Token::literal("api"),
                        Token::literal("database"),
                        Token::literal("cache"),
                    ]))),
                ),
                req(
                    "region",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("us-east-1"),
                        Token::literal("us-west-2"),
                        Token::literal("eu-west-1"),
                    ]))),
                ),
            ]),
        ),
    ])
}

pub fn coordinates() -> Template {
    Template::arr(
        Some(0),
        Some(0),
        Some(Template::nil()),
        vec![
            Template::float(Some(-180.0), Some(180.0)),
            Template::float(Some(-90.0), Some(90.0)),
        ],
        vec![Template::lit(Value::String("WGS84".to_string()))],
    )
}

pub fn address() -> Template {
    Template::obj(vec![
        req(
            "street",
            Template::str(Some(Token::list(vec![
                Token::char_range(48, 57, Some(3)),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("Main St"),
                    Token::literal("Oak Ave"),
                    Token::literal("Pine Rd"),
                    Token::literal("Elm St"),
                    Token::literal("Maple Dr"),
                ]),
            ]))),
        ),
        req(
            "city",
            Template::str(Some(Token::pick(vec![
                Token::literal("New York"),
                Token::literal("Los Angeles"),
                Token::literal("Chicago"),
                Token::literal("Houston"),
                Token::literal("Phoenix"),
                Token::literal("Philadelphia"),
                Token::literal("San Antonio"),
            ]))),
        ),
        req(
            "state",
            Template::str(Some(Token::pick(vec![
                Token::literal("NY"),
                Token::literal("CA"),
                Token::literal("IL"),
                Token::literal("TX"),
                Token::literal("AZ"),
                Token::literal("PA"),
            ]))),
        ),
        req(
            "country",
            Template::str(Some(Token::pick(vec![
                Token::literal("United States"),
                Token::literal("Canada"),
                Token::literal("Mexico"),
                Token::literal("United Kingdom"),
                Token::literal("Germany"),
                Token::literal("France"),
            ]))),
        ),
        req(
            "postalCode",
            Template::str(Some(Token::char_range(48, 57, Some(5)))),
        ),
        opt("coordinates", coordinates(), 0.3),
    ])
}

pub fn location() -> Template {
    Template::obj(vec![
        req(
            "name",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("Coffee Shop"),
                    Token::literal("Restaurant"),
                    Token::literal("Store"),
                    Token::literal("Office"),
                    Token::literal("Park"),
                ]),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("Downtown"),
                    Token::literal("Central"),
                    Token::literal("North"),
                    Token::literal("South"),
                    Token::literal("Main"),
                ]),
            ]))),
        ),
        req("address", address()),
        req("phone", Template::str(Some(token_phone()))),
        opt(
            "hours",
            Template::obj(vec![
                req(
                    "monday",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("9:00-17:00"),
                        Token::literal("8:00-18:00"),
                        Token::literal("closed"),
                    ]))),
                ),
                req(
                    "tuesday",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("9:00-17:00"),
                        Token::literal("8:00-18:00"),
                        Token::literal("closed"),
                    ]))),
                ),
                req(
                    "wednesday",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("9:00-17:00"),
                        Token::literal("8:00-18:00"),
                        Token::literal("closed"),
                    ]))),
                ),
                req(
                    "thursday",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("9:00-17:00"),
                        Token::literal("8:00-18:00"),
                        Token::literal("closed"),
                    ]))),
                ),
                req(
                    "friday",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("9:00-17:00"),
                        Token::literal("8:00-18:00"),
                        Token::literal("closed"),
                    ]))),
                ),
                req(
                    "saturday",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("10:00-16:00"),
                        Token::literal("9:00-15:00"),
                        Token::literal("closed"),
                    ]))),
                ),
                req(
                    "sunday",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("12:00-16:00"),
                        Token::literal("closed"),
                    ]))),
                ),
            ]),
            0.4,
        ),
    ])
}

pub fn transaction() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "id",
            Template::str(Some(Token::list(vec![
                Token::literal("txn_"),
                Token::char_range(97, 122, Some(8)),
                Token::char_range(48, 57, Some(8)),
            ]))),
        ),
        req("amount", Template::float(Some(0.01), Some(10_000.0))),
        req(
            "currency",
            Template::str(Some(Token::pick(vec![
                Token::literal("USD"),
                Token::literal("EUR"),
                Token::literal("GBP"),
                Token::literal("JPY"),
                Token::literal("CAD"),
                Token::literal("AUD"),
            ]))),
        ),
        req(
            "type",
            Template::str(Some(Token::pick(vec![
                Token::literal("debit"),
                Token::literal("credit"),
                Token::literal("transfer"),
                Token::literal("payment"),
                Token::literal("refund"),
            ]))),
        ),
        req(
            "status",
            Template::str(Some(Token::pick(vec![
                Token::literal("pending"),
                Token::literal("completed"),
                Token::literal("failed"),
                Token::literal("cancelled"),
            ]))),
        ),
        req(
            "fromAccount",
            Template::str(Some(Token::list(vec![
                Token::literal("acc_"),
                Token::char_range(48, 57, Some(12)),
            ]))),
        ),
        req(
            "toAccount",
            Template::str(Some(Token::list(vec![
                Token::literal("acc_"),
                Token::char_range(48, 57, Some(12)),
            ]))),
        ),
        req(
            "description",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("Payment to"),
                    Token::literal("Transfer from"),
                    Token::literal("Purchase at"),
                    Token::literal("Refund for"),
                ]),
                Token::literal(" "),
                Token::repeat(5, 20, Token::char_range(32, 126, None)),
            ]))),
        ),
        req(
            "timestamp",
            Template::int(Some(now.saturating_sub(86_400_000)), Some(now)),
        ),
        opt("fees", Template::float(Some(0.0), Some(50.0)), 0.3),
    ])
}

pub fn bank_account() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "accountNumber",
            Template::str(Some(Token::char_range(48, 57, Some(12)))),
        ),
        req(
            "routingNumber",
            Template::str(Some(Token::char_range(48, 57, Some(9)))),
        ),
        req(
            "accountType",
            Template::str(Some(Token::pick(vec![
                Token::literal("checking"),
                Token::literal("savings"),
                Token::literal("business"),
                Token::literal("credit"),
            ]))),
        ),
        req("balance", Template::float(Some(0.0), Some(100_000.0))),
        req(
            "currency",
            Template::str(Some(Token::pick(vec![
                Token::literal("USD"),
                Token::literal("EUR"),
                Token::literal("GBP"),
            ]))),
        ),
        req("isActive", Template::bool(None)),
        req("openedDate", Template::int(Some(946_684_800), Some(now))),
        req(
            "lastActivity",
            Template::int(Some(now.saturating_sub(2_592_000)), Some(now)),
        ),
    ])
}

pub fn social_post() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "id",
            Template::str(Some(Token::list(vec![
                Token::literal("post_"),
                Token::char_range(97, 122, Some(8)),
            ]))),
        ),
        req(
            "content",
            Template::str(Some(Token::repeat(
                10,
                280,
                Token::char_range(32, 126, None),
            ))),
        ),
        req(
            "author",
            Template::obj(vec![
                req("username", Template::str(Some(token_username()))),
                req(
                    "displayName",
                    Template::str(Some(Token::list(vec![
                        Token::pick(vec![
                            Token::literal("John"),
                            Token::literal("Jane"),
                            Token::literal("Alex"),
                            Token::literal("Sam"),
                            Token::literal("Chris"),
                        ]),
                        Token::literal(" "),
                        Token::pick(vec![
                            Token::literal("Smith"),
                            Token::literal("Doe"),
                            Token::literal("Johnson"),
                            Token::literal("Brown"),
                        ]),
                    ]))),
                ),
                req("verified", Template::bool(None)),
            ]),
        ),
        req("likes", Template::int(Some(0), Some(10_000))),
        req("shares", Template::int(Some(0), Some(1000))),
        req("comments", Template::int(Some(0), Some(500))),
        req(
            "hashtags",
            Template::arr(
                Some(0),
                Some(5),
                Some(Template::str(Some(Token::list(vec![
                    Token::literal("#"),
                    Token::repeat(3, 15, Token::char_range(97, 122, None)),
                ])))),
                vec![],
                vec![],
            ),
        ),
        req(
            "mentions",
            Template::arr(
                Some(0),
                Some(3),
                Some(Template::str(Some(Token::list(vec![
                    Token::literal("@"),
                    token_username(),
                ])))),
                vec![],
                vec![],
            ),
        ),
        req(
            "timestamp",
            Template::int(Some(now.saturating_sub(604_800_000)), Some(now)),
        ),
        opt(
            "media",
            Template::arr(
                Some(0),
                Some(4),
                Some(Template::obj(vec![
                    req(
                        "type",
                        Template::str(Some(Token::pick(vec![
                            Token::literal("image"),
                            Token::literal("video"),
                            Token::literal("gif"),
                        ]))),
                    ),
                    req("url", Template::str(Some(token_url()))),
                    opt(
                        "alt",
                        Template::str(Some(Token::repeat(5, 50, Token::char_range(32, 126, None)))),
                        0.7,
                    ),
                ])),
                vec![],
                vec![],
            ),
            0.4,
        ),
    ])
}

pub fn social_profile() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req("username", Template::str(Some(token_username()))),
        req(
            "displayName",
            Template::str(Some(Token::repeat(3, 30, Token::char_range(32, 126, None)))),
        ),
        opt(
            "bio",
            Template::str(Some(Token::repeat(
                10,
                160,
                Token::char_range(32, 126, None),
            ))),
            0.8,
        ),
        req("followers", Template::int(Some(0), Some(1_000_000))),
        req("following", Template::int(Some(0), Some(10_000))),
        req("posts", Template::int(Some(0), Some(50_000))),
        req("verified", Template::bool(None)),
        req("joinDate", Template::int(Some(946_684_800), Some(now))),
        opt(
            "location",
            Template::str(Some(Token::pick(vec![
                Token::literal("New York"),
                Token::literal("London"),
                Token::literal("Tokyo"),
                Token::literal("Berlin"),
                Token::literal("Sydney"),
            ]))),
            0.6,
        ),
        opt("website", Template::str(Some(token_url())), 0.3),
    ])
}

pub fn sensor_reading() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "sensorId",
            Template::str(Some(Token::list(vec![
                Token::literal("sensor_"),
                Token::char_range(65, 90, Some(2)),
                Token::char_range(48, 57, Some(6)),
            ]))),
        ),
        req(
            "deviceType",
            Template::str(Some(Token::pick(vec![
                Token::literal("temperature"),
                Token::literal("humidity"),
                Token::literal("pressure"),
                Token::literal("motion"),
                Token::literal("light"),
                Token::literal("sound"),
            ]))),
        ),
        req("value", Template::float(Some(-50.0), Some(150.0))),
        req(
            "unit",
            Template::str(Some(Token::pick(vec![
                Token::literal("celsius"),
                Token::literal("fahrenheit"),
                Token::literal("percent"),
                Token::literal("pascal"),
                Token::literal("lux"),
                Token::literal("decibel"),
            ]))),
        ),
        req(
            "timestamp",
            Template::int(Some(now.saturating_sub(3_600_000)), Some(now)),
        ),
        req(
            "location",
            Template::obj(vec![
                req(
                    "room",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("living_room"),
                        Token::literal("bedroom"),
                        Token::literal("kitchen"),
                        Token::literal("bathroom"),
                        Token::literal("office"),
                    ]))),
                ),
                req("floor", Template::int(Some(1), Some(10))),
                req(
                    "building",
                    Template::str(Some(Token::list(vec![
                        Token::literal("Building "),
                        Token::char_range(65, 90, Some(1)),
                    ]))),
                ),
            ]),
        ),
        req(
            "status",
            Template::str(Some(Token::pick(vec![
                Token::literal("online"),
                Token::literal("offline"),
                Token::literal("maintenance"),
                Token::literal("error"),
            ]))),
        ),
        opt("battery", Template::int(Some(0), Some(100)), 0.6),
    ])
}

pub fn iot_device() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "deviceId",
            Template::str(Some(Token::list(vec![
                Token::literal("iot_"),
                Token::char_range(97, 122, Some(4)),
                Token::char_range(48, 57, Some(8)),
            ]))),
        ),
        req(
            "name",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("Smart"),
                    Token::literal("Connected"),
                    Token::literal("Wireless"),
                    Token::literal("Digital"),
                ]),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("Thermostat"),
                    Token::literal("Camera"),
                    Token::literal("Lock"),
                    Token::literal("Light"),
                    Token::literal("Sensor"),
                ]),
            ]))),
        ),
        req(
            "manufacturer",
            Template::str(Some(Token::pick(vec![
                Token::literal("SmartHome Inc"),
                Token::literal("IoT Solutions"),
                Token::literal("TechDevice Co"),
                Token::literal("ConnectCorp"),
            ]))),
        ),
        req(
            "model",
            Template::str(Some(Token::list(vec![
                Token::char_range(65, 90, Some(2)),
                Token::literal("-"),
                Token::char_range(48, 57, Some(4)),
            ]))),
        ),
        req(
            "firmwareVersion",
            Template::str(Some(Token::list(vec![
                Token::char_range(49, 57, Some(1)),
                Token::literal("."),
                Token::char_range(48, 57, Some(1)),
                Token::literal("."),
                Token::char_range(48, 57, Some(1)),
            ]))),
        ),
        req(
            "ipAddress",
            Template::str(Some(Token::list(vec![
                Token::literal("192.168."),
                Token::char_range(48, 57, Some(1)),
                Token::literal("."),
                Token::char_range(48, 57, Some(3)),
            ]))),
        ),
        req(
            "macAddress",
            Template::str(Some(Token::list(vec![
                Token::char_range(48, 57, Some(2)),
                Token::literal(":"),
                Token::char_range(48, 57, Some(2)),
                Token::literal(":"),
                Token::char_range(48, 57, Some(2)),
                Token::literal(":"),
                Token::char_range(48, 57, Some(2)),
                Token::literal(":"),
                Token::char_range(48, 57, Some(2)),
                Token::literal(":"),
                Token::char_range(48, 57, Some(2)),
            ]))),
        ),
        req(
            "lastSeen",
            Template::int(Some(now.saturating_sub(86_400_000)), Some(now)),
        ),
        req(
            "sensors",
            Template::arr(Some(1), Some(4), Some(sensor_reading()), vec![], vec![]),
        ),
    ])
}

pub fn patient() -> Template {
    Template::obj(vec![
        req(
            "patientId",
            Template::str(Some(Token::list(vec![
                Token::literal("PAT-"),
                Token::char_range(48, 57, Some(8)),
            ]))),
        ),
        req(
            "firstName",
            Template::str(Some(Token::pick(vec![
                Token::literal("John"),
                Token::literal("Jane"),
                Token::literal("Michael"),
                Token::literal("Sarah"),
                Token::literal("David"),
                Token::literal("Emily"),
                Token::literal("James"),
                Token::literal("Lisa"),
            ]))),
        ),
        req(
            "lastName",
            Template::str(Some(Token::pick(vec![
                Token::literal("Smith"),
                Token::literal("Johnson"),
                Token::literal("Williams"),
                Token::literal("Brown"),
                Token::literal("Jones"),
                Token::literal("Garcia"),
                Token::literal("Miller"),
                Token::literal("Davis"),
            ]))),
        ),
        req(
            "dateOfBirth",
            Template::int(Some(157_766_400), Some(1_009_843_200)),
        ),
        req(
            "gender",
            Template::str(Some(Token::pick(vec![
                Token::literal("male"),
                Token::literal("female"),
                Token::literal("non-binary"),
                Token::literal("prefer-not-to-say"),
            ]))),
        ),
        req(
            "bloodType",
            Template::str(Some(Token::pick(vec![
                Token::literal("A+"),
                Token::literal("A-"),
                Token::literal("B+"),
                Token::literal("B-"),
                Token::literal("AB+"),
                Token::literal("AB-"),
                Token::literal("O+"),
                Token::literal("O-"),
            ]))),
        ),
        req(
            "allergies",
            Template::arr(
                Some(0),
                Some(5),
                Some(Template::str(Some(Token::pick(vec![
                    Token::literal("peanuts"),
                    Token::literal("shellfish"),
                    Token::literal("dairy"),
                    Token::literal("gluten"),
                    Token::literal("penicillin"),
                    Token::literal("latex"),
                ])))),
                vec![],
                vec![],
            ),
        ),
        req(
            "emergencyContact",
            Template::obj(vec![
                req(
                    "name",
                    Template::str(Some(Token::list(vec![
                        Token::pick(vec![
                            Token::literal("John"),
                            Token::literal("Jane"),
                            Token::literal("Michael"),
                            Token::literal("Sarah"),
                        ]),
                        Token::literal(" "),
                        Token::pick(vec![
                            Token::literal("Smith"),
                            Token::literal("Johnson"),
                            Token::literal("Williams"),
                            Token::literal("Brown"),
                        ]),
                    ]))),
                ),
                req(
                    "relationship",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("spouse"),
                        Token::literal("parent"),
                        Token::literal("sibling"),
                        Token::literal("child"),
                        Token::literal("friend"),
                    ]))),
                ),
                req("phone", Template::str(Some(token_phone()))),
            ]),
        ),
        opt(
            "insurance",
            Template::obj(vec![
                req(
                    "provider",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("HealthCare Plus"),
                        Token::literal("MedInsure"),
                        Token::literal("WellnessCare"),
                        Token::literal("LifeHealth"),
                    ]))),
                ),
                req(
                    "policyNumber",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(65, 90, Some(3)),
                        Token::char_range(48, 57, Some(9)),
                    ]))),
                ),
                req(
                    "groupNumber",
                    Template::str(Some(Token::char_range(48, 57, Some(6)))),
                ),
            ]),
            0.9,
        ),
    ])
}

pub fn medical_record() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "recordId",
            Template::str(Some(Token::list(vec![
                Token::literal("MED-"),
                Token::char_range(48, 57, Some(10)),
            ]))),
        ),
        req(
            "patientId",
            Template::str(Some(Token::list(vec![
                Token::literal("PAT-"),
                Token::char_range(48, 57, Some(8)),
            ]))),
        ),
        req(
            "visitDate",
            Template::int(Some(now.saturating_sub(31_536_000)), Some(now)),
        ),
        req(
            "provider",
            Template::obj(vec![
                req(
                    "name",
                    Template::str(Some(Token::list(vec![
                        Token::literal("Dr. "),
                        Token::pick(vec![
                            Token::literal("John"),
                            Token::literal("Jane"),
                            Token::literal("Michael"),
                            Token::literal("Sarah"),
                        ]),
                        Token::literal(" "),
                        Token::pick(vec![
                            Token::literal("Smith"),
                            Token::literal("Johnson"),
                            Token::literal("Williams"),
                        ]),
                    ]))),
                ),
                req(
                    "specialty",
                    Template::str(Some(Token::pick(vec![
                        Token::literal("cardiology"),
                        Token::literal("neurology"),
                        Token::literal("pediatrics"),
                        Token::literal("orthopedics"),
                        Token::literal("dermatology"),
                    ]))),
                ),
                req(
                    "license",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(65, 90, Some(2)),
                        Token::char_range(48, 57, Some(8)),
                    ]))),
                ),
            ]),
        ),
        req(
            "diagnosis",
            Template::arr(
                Some(1),
                Some(3),
                Some(Template::str(Some(Token::pick(vec![
                    Token::literal("hypertension"),
                    Token::literal("diabetes"),
                    Token::literal("asthma"),
                    Token::literal("arthritis"),
                    Token::literal("migraine"),
                ])))),
                vec![],
                vec![],
            ),
        ),
        req(
            "medications",
            Template::arr(
                Some(0),
                Some(5),
                Some(Template::obj(vec![
                    req(
                        "name",
                        Template::str(Some(Token::pick(vec![
                            Token::literal("Lisinopril"),
                            Token::literal("Metformin"),
                            Token::literal("Albuterol"),
                            Token::literal("Ibuprofen"),
                            Token::literal("Atorvastatin"),
                        ]))),
                    ),
                    req(
                        "dosage",
                        Template::str(Some(Token::list(vec![
                            Token::char_range(48, 57, Some(2)),
                            Token::literal("mg"),
                        ]))),
                    ),
                    req(
                        "frequency",
                        Template::str(Some(Token::pick(vec![
                            Token::literal("once daily"),
                            Token::literal("twice daily"),
                            Token::literal("three times daily"),
                            Token::literal("as needed"),
                        ]))),
                    ),
                ])),
                vec![],
                vec![],
            ),
        ),
        opt(
            "vitals",
            Template::obj(vec![
                req(
                    "bloodPressure",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(49, 57, Some(1)),
                        Token::char_range(48, 57, Some(2)),
                        Token::literal("/"),
                        Token::char_range(48, 57, Some(2)),
                    ]))),
                ),
                req("heartRate", Template::int(Some(60), Some(100))),
                req("temperature", Template::float(Some(96.0), Some(104.0))),
                req("weight", Template::float(Some(100.0), Some(300.0))),
            ]),
            0.8,
        ),
    ])
}

pub fn student() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "studentId",
            Template::str(Some(Token::list(vec![
                Token::literal("STU-"),
                Token::char_range(48, 57, Some(7)),
            ]))),
        ),
        req(
            "firstName",
            Template::str(Some(Token::pick(vec![
                Token::literal("Alex"),
                Token::literal("Sam"),
                Token::literal("Jordan"),
                Token::literal("Casey"),
                Token::literal("Taylor"),
                Token::literal("Morgan"),
                Token::literal("Riley"),
                Token::literal("Avery"),
            ]))),
        ),
        req(
            "lastName",
            Template::str(Some(Token::pick(vec![
                Token::literal("Anderson"),
                Token::literal("Brown"),
                Token::literal("Clark"),
                Token::literal("Davis"),
                Token::literal("Evans"),
                Token::literal("Foster"),
                Token::literal("Green"),
                Token::literal("Hill"),
            ]))),
        ),
        req("email", Template::str(Some(token_email()))),
        req(
            "grade",
            Template::str(Some(Token::pick(vec![
                Token::literal("9th"),
                Token::literal("10th"),
                Token::literal("11th"),
                Token::literal("12th"),
                Token::literal("Freshman"),
                Token::literal("Sophomore"),
                Token::literal("Junior"),
                Token::literal("Senior"),
            ]))),
        ),
        opt(
            "major",
            Template::str(Some(Token::pick(vec![
                Token::literal("Computer Science"),
                Token::literal("Biology"),
                Token::literal("Mathematics"),
                Token::literal("English"),
                Token::literal("History"),
                Token::literal("Physics"),
                Token::literal("Chemistry"),
            ]))),
            0.7,
        ),
        req("gpa", Template::float(Some(2.0), Some(4.0))),
        req(
            "enrollmentDate",
            Template::int(Some(1_567_296_000), Some(now)),
        ),
    ])
}

pub fn course() -> Template {
    Template::obj(vec![
        req(
            "courseId",
            Template::str(Some(Token::list(vec![
                Token::char_range(65, 90, Some(3)),
                Token::literal("-"),
                Token::char_range(48, 57, Some(3)),
            ]))),
        ),
        req(
            "title",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("Introduction to"),
                    Token::literal("Advanced"),
                    Token::literal("Fundamentals of"),
                    Token::literal("Applied"),
                ]),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("Computer Science"),
                    Token::literal("Mathematics"),
                    Token::literal("Biology"),
                    Token::literal("Chemistry"),
                    Token::literal("Physics"),
                    Token::literal("History"),
                    Token::literal("Literature"),
                ]),
            ]))),
        ),
        req("credits", Template::int(Some(1), Some(6))),
        req(
            "instructor",
            Template::obj(vec![
                req(
                    "name",
                    Template::str(Some(Token::list(vec![
                        Token::literal("Prof. "),
                        Token::pick(vec![
                            Token::literal("John"),
                            Token::literal("Jane"),
                            Token::literal("Michael"),
                            Token::literal("Sarah"),
                        ]),
                        Token::literal(" "),
                        Token::pick(vec![
                            Token::literal("Smith"),
                            Token::literal("Johnson"),
                            Token::literal("Williams"),
                        ]),
                    ]))),
                ),
                req("email", Template::str(Some(token_email()))),
                req(
                    "office",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(65, 90, Some(1)),
                        Token::literal("-"),
                        Token::char_range(48, 57, Some(3)),
                    ]))),
                ),
            ]),
        ),
        req(
            "schedule",
            Template::obj(vec![
                req(
                    "days",
                    Template::arr(
                        Some(1),
                        Some(3),
                        Some(Template::str(Some(Token::pick(vec![
                            Token::literal("Monday"),
                            Token::literal("Tuesday"),
                            Token::literal("Wednesday"),
                            Token::literal("Thursday"),
                            Token::literal("Friday"),
                        ])))),
                        vec![],
                        vec![],
                    ),
                ),
                req(
                    "time",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(48, 57, Some(2)),
                        Token::literal(":"),
                        Token::pick(vec![Token::literal("00"), Token::literal("30")]),
                        Token::literal("-"),
                        Token::char_range(48, 57, Some(2)),
                        Token::literal(":"),
                        Token::pick(vec![Token::literal("00"), Token::literal("30")]),
                    ]))),
                ),
                req(
                    "room",
                    Template::str(Some(Token::list(vec![
                        Token::char_range(65, 90, Some(1)),
                        Token::literal("-"),
                        Token::char_range(48, 57, Some(3)),
                    ]))),
                ),
            ]),
        ),
        req("capacity", Template::int(Some(15), Some(200))),
        req("enrolled", Template::int(Some(5), Some(180))),
    ])
}

pub fn grade() -> Template {
    let now = now_millis();
    Template::obj(vec![
        req(
            "studentId",
            Template::str(Some(Token::list(vec![
                Token::literal("STU-"),
                Token::char_range(48, 57, Some(7)),
            ]))),
        ),
        req(
            "courseId",
            Template::str(Some(Token::list(vec![
                Token::char_range(65, 90, Some(3)),
                Token::literal("-"),
                Token::char_range(48, 57, Some(3)),
            ]))),
        ),
        req(
            "semester",
            Template::str(Some(Token::pick(vec![
                Token::literal("Fall 2023"),
                Token::literal("Spring 2024"),
                Token::literal("Summer 2024"),
                Token::literal("Fall 2024"),
            ]))),
        ),
        req(
            "assignments",
            Template::arr(
                Some(3),
                Some(8),
                Some(Template::obj(vec![
                    req(
                        "name",
                        Template::str(Some(Token::list(vec![
                            Token::pick(vec![
                                Token::literal("Assignment"),
                                Token::literal("Quiz"),
                                Token::literal("Exam"),
                                Token::literal("Project"),
                                Token::literal("Lab"),
                            ]),
                            Token::literal(" "),
                            Token::char_range(48, 57, Some(1)),
                        ]))),
                    ),
                    req("score", Template::float(Some(0.0), Some(100.0))),
                    req("maxScore", Template::lit(Value::Number(100.into()))),
                    req(
                        "dueDate",
                        Template::int(
                            Some(now.saturating_sub(7_776_000)),
                            Some(now.saturating_add(7_776_000)),
                        ),
                    ),
                    req("submitted", Template::bool(None)),
                ])),
                vec![],
                vec![],
            ),
        ),
        req(
            "finalGrade",
            Template::str(Some(Token::pick(vec![
                Token::literal("A+"),
                Token::literal("A"),
                Token::literal("A-"),
                Token::literal("B+"),
                Token::literal("B"),
                Token::literal("B-"),
                Token::literal("C+"),
                Token::literal("C"),
                Token::literal("C-"),
                Token::literal("D+"),
                Token::literal("D"),
                Token::literal("F"),
            ]))),
        ),
        req("gpa", Template::float(Some(0.0), Some(4.0))),
    ])
}

pub fn empty_structures() -> Template {
    Template::obj(vec![
        req("emptyObject", Template::obj(vec![])),
        req(
            "emptyArray",
            Template::arr(Some(0), Some(0), None, vec![], vec![]),
        ),
        req("emptyString", Template::lit(Value::String(String::new()))),
        req("nullValue", Template::nil()),
        req("zeroNumber", Template::lit(Value::Number(0.into()))),
        req("falseBool", Template::lit(Value::Bool(false))),
    ])
}

pub fn unicode_text() -> Template {
    Template::obj(vec![
        req(
            "ascii",
            Template::str(Some(Token::repeat(5, 15, Token::char_range(32, 126, None)))),
        ),
        req(
            "latin",
            Template::str(Some(Token::repeat(
                5,
                15,
                Token::char_range(160, 255, None),
            ))),
        ),
        req(
            "emoji",
            Template::str(Some(Token::repeat(
                1,
                5,
                Token::char_range(0x1F600, 0x1F64F, None),
            ))),
        ),
        req(
            "chinese",
            Template::str(Some(Token::repeat(
                3,
                8,
                Token::char_range(0x4E00, 0x9FFF, None),
            ))),
        ),
        req(
            "arabic",
            Template::str(Some(Token::repeat(
                3,
                8,
                Token::char_range(0x0600, 0x06FF, None),
            ))),
        ),
        req(
            "mixed",
            Template::str(Some(Token::list(vec![
                Token::repeat(2, 5, Token::char_range(65, 90, None)),
                Token::literal(" "),
                Token::char_range(0x1F600, 0x1F64F, None),
                Token::literal(" "),
                Token::repeat(2, 5, Token::char_range(0x4E00, 0x9FFF, None)),
            ]))),
        ),
    ])
}

pub fn large_numbers() -> Template {
    Template::obj(vec![
        req(
            "maxSafeInteger",
            Template::lit(json!(9_007_199_254_740_991_i64)),
        ),
        req(
            "minSafeInteger",
            Template::lit(json!(-9_007_199_254_740_991_i64)),
        ),
        req(
            "largeFloat",
            Template::float(Some(1e10_f64), Some(1e15_f64)),
        ),
        req(
            "smallFloat",
            Template::float(Some(1e-10_f64), Some(1e-5_f64)),
        ),
        req(
            "preciseDecimal",
            Template::float(Some(0.000001_f64), Some(0.999999_f64)),
        ),
        req("scientificNotation", Template::lit(json!(1.23e-45_f64))),
    ])
}

pub fn performance_test() -> Template {
    Template::arr(
        Some(100),
        Some(1000),
        Some(Template::obj(vec![
            req("id", Template::int(Some(1), Some(1_000_000))),
            req(
                "data",
                Template::str(Some(Token::repeat(
                    50,
                    200,
                    Token::char_range(32, 126, None),
                ))),
            ),
            req(
                "nested",
                Template::obj(vec![req(
                    "level1",
                    Template::obj(vec![req(
                        "level2",
                        Template::obj(vec![req(
                            "level3",
                            Template::arr(
                                Some(5),
                                Some(10),
                                Some(Template::int(None, None)),
                                vec![],
                                vec![],
                            ),
                        )]),
                    )]),
                )]),
            ),
        ])),
        vec![],
        vec![],
    )
}

pub fn mixed_types() -> Template {
    Template::or(vec![
        Template::str(None),
        Template::int(None, None),
        Template::float(None, None),
        Template::bool(None),
        Template::nil(),
        Template::arr(Some(1), Some(3), Some(Template::str(None)), vec![], vec![]),
        Template::obj(vec![
            req("key1", Template::str(None)),
            req("key2", Template::int(None, None)),
        ]),
    ])
}

pub fn load_test_user() -> Template {
    Template::obj(vec![
        req("id", Template::int(Some(1), Some(10_000))),
        req(
            "name",
            Template::str(Some(Token::list(vec![
                Token::pick(vec![
                    Token::literal("John"),
                    Token::literal("Jane"),
                    Token::literal("Bob"),
                    Token::literal("Alice"),
                    Token::literal("Charlie"),
                ]),
                Token::literal(" "),
                Token::pick(vec![
                    Token::literal("Doe"),
                    Token::literal("Smith"),
                    Token::literal("Johnson"),
                    Token::literal("Brown"),
                ]),
            ]))),
        ),
        req(
            "email",
            Template::str(Some(Token::list(vec![
                Token::repeat(3, 10, Token::char_range(97, 122, None)),
                Token::literal("@test.com"),
            ]))),
        ),
        req("age", Template::int(Some(18), Some(65))),
        req("active", Template::bool(None)),
    ])
}

pub fn all_examples() -> Template {
    Template::or(vec![
        user_profile(),
        user_basic(),
        api_response(),
        api_response_detailed(),
        service_config(),
        product(),
        order(),
        user_token(),
        user_role(),
        log_entry(),
        metric_data(),
        address(),
        location(),
        transaction(),
        bank_account(),
        social_post(),
        social_profile(),
        sensor_reading(),
        iot_device(),
        patient(),
        medical_record(),
        student(),
        course(),
        grade(),
        empty_structures(),
        unicode_text(),
        large_numbers(),
        performance_test(),
        mixed_types(),
        load_test_user(),
        Template::recursive(tree),
        Template::recursive(comment),
    ])
}

pub fn gen_user() -> Value {
    TemplateJson::gen(Some(user_profile()), None)
}

pub fn gen_user_basic() -> Value {
    TemplateJson::gen(Some(user_basic()), None)
}

pub fn gen_address() -> Value {
    TemplateJson::gen(Some(address()), None)
}

pub fn gen_product() -> Value {
    TemplateJson::gen(Some(product()), None)
}

pub fn gen_order() -> Value {
    TemplateJson::gen(Some(order()), None)
}

pub fn gen_transaction() -> Value {
    TemplateJson::gen(Some(transaction()), None)
}

pub fn gen_bank_account() -> Value {
    TemplateJson::gen(Some(bank_account()), None)
}

pub fn gen_social_post() -> Value {
    TemplateJson::gen(Some(social_post()), None)
}

pub fn gen_social_profile() -> Value {
    TemplateJson::gen(Some(social_profile()), None)
}

pub fn gen_location() -> Value {
    TemplateJson::gen(Some(location()), None)
}

pub fn gen_api_response() -> Value {
    TemplateJson::gen(Some(api_response()), None)
}

pub fn gen_api_response_detailed() -> Value {
    TemplateJson::gen(Some(api_response_detailed()), None)
}

pub fn gen_service_config() -> Value {
    TemplateJson::gen(Some(service_config()), None)
}

pub fn gen_patient() -> Value {
    TemplateJson::gen(Some(patient()), None)
}

pub fn gen_medical_record() -> Value {
    TemplateJson::gen(Some(medical_record()), None)
}

pub fn gen_student() -> Value {
    TemplateJson::gen(Some(student()), None)
}

pub fn gen_course() -> Value {
    TemplateJson::gen(Some(course()), None)
}

pub fn gen_sensor_reading() -> Value {
    TemplateJson::gen(Some(sensor_reading()), None)
}

pub fn gen_iot_device() -> Value {
    TemplateJson::gen(Some(iot_device()), None)
}

pub fn gen_log_entry() -> Value {
    TemplateJson::gen(Some(log_entry()), None)
}

pub fn gen_metric_data() -> Value {
    TemplateJson::gen(Some(metric_data()), None)
}

pub fn gen_random_example() -> Value {
    TemplateJson::gen(Some(all_examples()), None)
}
