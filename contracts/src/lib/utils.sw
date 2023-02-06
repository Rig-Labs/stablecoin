library utils;

function compute_CR(coll: u64, debt:  u64, price:  u64) -> uint64 {
    if (debt > 0) {
        (coll * price) / debt
    }
    // Return the maximal value for u64 if the Trove has a debt of 0. Represents "infinite" CR.
    else { // if (_debt == 0)
        u64::MAX - 1
    }
}

