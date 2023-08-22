use std::path::{Path, PathBuf};

const DECIMAL_PRECISION: u64 = 1_000_000_000;

// 0.5% min borrow fee
pub fn with_min_borrow_fee(debt: u64) -> u64 {
    let net_debt = debt * 1_005 / 1_000;
    return net_debt;
}

// 1% min redemption fee
pub fn with_min_redemption_fee(amount: u64) -> u64 {
    let amount_with_fee = amount * 1_010 / 1_000;
    return amount_with_fee;
}

pub fn calculate_icr(coll: u64, debt: u64) -> u64 {
    let icr = coll as u128 * DECIMAL_PRECISION as u128 / debt as u128;
    return icr.try_into().unwrap();
}

pub fn calculate_cr(price: u64, coll: u64, debt: u64) -> u64 {
    let cr = price as u128 * coll as u128 / debt as u128;
    return cr.try_into().unwrap();
}

pub fn with_liquidation_penalty(amount: u64) -> u64 {
    let amount_with_penalty = amount * 1_10 / 1_00;
    return amount_with_penalty;
}

pub fn resolve_relative_path(path: &str) -> String {
    let mut resolved = PathBuf::new();

    let mut components = Path::new(path).components().peekable();

    // Handle leading `../` segments
    while let Some(component) = components.peek() {
        if *component == std::path::Component::ParentDir {
            resolved.push(component.as_os_str());
            components.next();
        } else {
            break;
        }
    }

    // Append remaining path components
    for component in components {
        resolved.push(component.as_os_str());
    }

    // Return absolute path as a string
    let mut resolved_str: String;
    if resolved.is_relative() {
        let mut abs_path = std::env::current_dir().unwrap();
        abs_path.push(resolved);
        resolved_str = abs_path.to_string_lossy().to_string()
    } else {
        resolved_str = resolved.to_string_lossy().to_string()
    }

    while let Some(pos) = resolved_str.find("test-utils/../../") {
        resolved_str.replace_range(pos..(pos + "test-utils/../../".len()), "");
    }

    // Return modified resolved path as a string
    resolved_str
}
