use steel::rvals::SteelVal;

/// Convert a Steel s-expression (SteelVal) to HTML string
pub fn sexp_to_html(sexp: &SteelVal) -> String {
    match sexp {
        SteelVal::StringV(s) => s.to_string(),
        SteelVal::IntV(n) => n.to_string(),
        SteelVal::NumV(n) => n.to_string(),
        SteelVal::BoolV(b) => b.to_string(),
        SteelVal::SymbolV(s) => s.to_string(),
        SteelVal::ListV(list) => {
            if list.is_empty() {
                return String::new();
            }

            // Convert list to Vec for easier manipulation
            let items: Vec<&SteelVal> = list.iter().collect();

            // First element should be the tag name
            if let Some(SteelVal::SymbolV(tag)) = items.get(0) {
                let tag_name = tag.to_string();

                // Check if second element is attributes (list of lists)
                let (has_attrs, attr_html) = if items.len() > 1 {
                    if let Some(SteelVal::ListV(maybe_attrs)) = items.get(1) {
                        let attrs_vec: Vec<&SteelVal> = maybe_attrs.iter().collect();
                        if !attrs_vec.is_empty() && is_attr_list(&attrs_vec) {
                            (true, attrs_to_html(&attrs_vec))
                        } else {
                            (false, String::new())
                        }
                    } else {
                        (false, String::new())
                    }
                } else {
                    (false, String::new())
                };

                // Get children (skip tag and optionally attributes)
                let children_start = if has_attrs { 2 } else { 1 };
                let children: Vec<String> = items[children_start..]
                    .iter()
                    .map(|child| sexp_to_html(child))
                    .collect();

                format!("<{}{}>{}</{}>", tag_name, attr_html, children.join(""), tag_name)
            } else {
                // Not a valid HTML s-exp, just concatenate children
                items.iter().map(|child| sexp_to_html(child)).collect::<Vec<_>>().join("")
            }
        }
        SteelVal::VectorV(vec) => {
            vec.iter().map(|v| sexp_to_html(v)).collect::<Vec<_>>().join("")
        }
        _ => String::new(),
    }
}

/// Check if a list looks like an attribute list
fn is_attr_list(list: &[&SteelVal]) -> bool {
    if list.is_empty() {
        return true; // Empty attribute list is valid
    }

    // Attribute list should be a list of lists, where each inner list has 2 elements (key value)
    list.iter().all(|item| {
        if let SteelVal::ListV(pair) = item {
            pair.len() == 2
        } else {
            false
        }
    })
}

/// Convert attribute list to HTML attributes string
fn attrs_to_html(attrs: &[&SteelVal]) -> String {
    let attr_strs: Vec<String> = attrs
        .iter()
        .filter_map(|attr| {
            if let SteelVal::ListV(pair) = attr {
                if pair.len() == 2 {
                    let pair_vec: Vec<&SteelVal> = pair.iter().collect();
                    let key = match pair_vec[0] {
                        SteelVal::SymbolV(s) => s.to_string(),
                        SteelVal::StringV(s) => s.to_string(),
                        _ => return None,
                    };
                    let value = match pair_vec[1] {
                        SteelVal::StringV(s) => s.to_string(),
                        SteelVal::SymbolV(s) => s.to_string(),
                        SteelVal::IntV(n) => n.to_string(),
                        _ => return None,
                    };
                    Some(format!("{}=\"{}\"", key, value))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    if attr_strs.is_empty() {
        String::new()
    } else {
        format!(" {}", attr_strs.join(" "))
    }
}
