use limit_muisqa::{load_dataset, MultiIntentQuestion, SubIntent, evaluate};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let entries = load_dataset("data/muisqa_sample.json")?;
    let entry = &entries[0];

    let mut q = MultiIntentQuestion {
        id: Uuid::new_v4(),
        text: entry.question.clone(),
        domain: entry.domain.clone(),
        intents: entry.sub_intents.iter().map(|s| SubIntent {
            id: Uuid::new_v4(),
            text: s.clone(),
            retrieved_evidence: vec!["dummy evidence".into()],
        }).collect(),
    };

    let metrics = evaluate(&q, &entry.answers);
    println!("Metrics: IRR={}, AA={}, AC={}, Entropy={}", metrics.irr, metrics.aa, metrics.ac, metrics.entropy);

    Ok(())
}
