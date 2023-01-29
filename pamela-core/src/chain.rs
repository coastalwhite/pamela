type Chain = Vec<bool>;

fn exec_chain(rules: &[PamRule]) -> bool {
    let chain = Chain(Vec::with_capacity(rules.len()));

    match PamRule {
        Binding => {
            chain.iter().all(|b| b);
        }
    }

}