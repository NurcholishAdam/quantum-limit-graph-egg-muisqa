use crate::question::MultiIntentQuestion;

pub struct Metrics {
    pub irr: f32,   // Information Recall Rate
    pub aa: f32,    // Answer Accuracy
    pub ac: f32,    // Answer Coverage
    pub entropy: f32,
}

pub fn evaluate(q: &MultiIntentQuestion, gold_answers: &[String]) -> Metrics {
    // stubbed metrics — extend with embeddings later
    let irr = q.intents.iter().map(|i| i.retrieved_evidence.len()).sum::<usize>() as f32;
    let aa = 0.8; // placeholder
    let ac = 0.7; // placeholder
    let entropy = 0.5; // placeholder
    Metrics { irr, aa, ac, entropy }
}
