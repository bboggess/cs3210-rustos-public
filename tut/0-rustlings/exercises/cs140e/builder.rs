// FIXME: Make me pass! Diff budget: 30 lines.

#[derive(Default)]
struct Builder {
    string: Option<String>,
    number: Option<usize>,
}

impl Builder {
    fn string(self, s: impl ToString) -> Self {
        Builder {
            string: Some(s.to_string()),
            number: self.number,
        }
    }

    fn number(self, number: usize) -> Self {
        Builder {
            string: self.string,
            number: Some(number),
        }
    }
}

impl ToString for Builder {
    // Implement the trait
    fn to_string(&self) -> String {
        let s = self.string.as_ref().map(|s| s.as_str()).unwrap_or_default();
        let n = &self
            .number
            .as_ref()
            .map(|n| n.to_string())
            .unwrap_or(String::from(""));
        let sep = if self.string.is_none() || self.number.is_none() {
            ""
        } else {
            " "
        };

        [s, n].join(sep)
    }
}

// Do not modify this function.
#[test]
fn builder() {
    let empty = Builder::default().to_string();
    assert_eq!(empty, "");

    let just_str = Builder::default().string("hi").to_string();
    assert_eq!(just_str, "hi");

    let just_num = Builder::default().number(254).to_string();
    assert_eq!(just_num, "254");

    let a = Builder::default()
        .string("hello, world!")
        .number(200)
        .to_string();

    assert_eq!(a, "hello, world! 200");

    let b = Builder::default()
        .string("hello, world!")
        .number(200)
        .string("bye now!")
        .to_string();

    assert_eq!(b, "bye now! 200");

    let c = Builder::default()
        .string("heap!".to_owned())
        .to_string();

    assert_eq!(c, "heap!");
}
