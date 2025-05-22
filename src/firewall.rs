
pub struct FirewallRule {
    pub to: String,
    pub from: String
}

pub struct Firewall {
    pub rules: Vec<FirewallRule>
}

impl Firewall {

    pub fn new(raw: &str) -> Self {
        
        let mut rules: Vec<FirewallRule> = Vec::new();

        for rule_line in raw.lines() {
            let rule_split: Vec<&str> = rule_line.split('=').collect::<Vec<&str>>();
            if rule_split.len() == 2 {
                rules.push(FirewallRule { 
                    to: rule_split.get(1).unwrap().trim().to_string(), 
                    from: rule_split.get(0).unwrap().trim().to_string() 
                });
            }
        }

        Self { rules }

    }

    pub fn apply(&self, path: &str) -> Option<String> {

        if let Some((path_from, path_to)) = path.split_once('/') {
            for rule in &self.rules {
                if rule.from.eq_ignore_ascii_case(&path_from) {
                    return Some(rule.to.clone() + "/" + path_to);
                }
            }
        }

        None

    }

}
