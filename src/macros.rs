macro_rules! color {
    ($i:ident, $t:expr) => {
        match $t {
            Transaction::Add => {
                $i.green()
            },
            Transaction::Update => {
                $i.blue()
            },
            Transaction::Remove => {
                $i.red()
            },
            Transaction::Ignore => {
                $i.white()
            },
            Transaction::Existing => {
                $i.purple()
            }
        }
    };
}

macro_rules! state {
    ($i:expr) => {
        match $i {
            State::Enabled => { "Enabled" },
            State::OptionalDisabled => { "Optional (Disabled)" },
            State::OptionalEnabled => { "Optional (Enabled)" },
            _ => { "" }
        }
    };
}
