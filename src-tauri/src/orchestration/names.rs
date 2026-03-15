pub fn generate_agent_name() -> String {
    let adjectives = [
        "swift", "keen", "bold", "iron", "bright", "deep", "calm", "sharp", "sage", "nova",
        "arc", "flux", "pulse", "forge", "drift", "spark", "blaze", "frost", "echo", "apex",
        "onyx", "jade", "zinc", "cobalt", "ember", "lunar", "solar", "cyber", "nexus", "prism",
    ];
    let nouns = [
        "wolf", "hawk", "fox", "bear", "lynx", "pike", "crow", "wren", "moth", "wasp",
        "oak", "elm", "fern", "reed", "vine", "bolt", "core", "node", "mesh", "arch",
        "atlas", "titan", "scout", "pilot", "agent", "proxy", "relay", "shard", "vault", "cache",
    ];

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    let hash = hasher.finish();

    let adj = adjectives[(hash as usize) % adjectives.len()];
    let noun = nouns[((hash >> 16) as usize) % nouns.len()];
    let num = (hash % 100) as u32;

    format!("{}-{}-{:02}", adj, noun, num)
}
