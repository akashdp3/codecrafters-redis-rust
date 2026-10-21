#[derive(Debug)]
pub(crate) struct AclUser {
    enabled: bool,
    flags: Vec<String>,
    passwords: Vec<String>,
}

impl Default for AclUser {
    fn default() -> Self {
        Self {
            enabled: true,
            flags: vec!["nopass".to_string()],
            passwords: vec![],
        }
    }
}

impl AclUser {
    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn flags(&self) -> &Vec<String> {
        &self.flags
    }

    pub(crate) fn passwords(&self) -> &Vec<String> {
        &self.passwords
    }

    pub(crate) fn _add_flag(&mut self, flag: String) -> &mut Self {
        self.flags.push(flag);

        self
    }

    pub(crate) fn remove_flag(&mut self, flag: String) -> &mut Self {
        self.flags.retain(|f| *f != flag);

        self
    }

    pub(crate) fn add_password(&mut self, password: String) -> &mut Self {
        self.passwords.push(password);

        self
    }
}
