#[cfg(test)]
#[cfg(feature = "it")]
mod tests {
    use anyhow::Context;
    use libakaza::lm::base::{SystemBigramLM, SystemUnigramLM};

    use libakaza::lm::system_bigram::MarisaSystemBigramLM;
    use libakaza::lm::system_unigram_lm::MarisaSystemUnigramLM;

    fn basedir() -> String {
        env!("CARGO_MANIFEST_DIR").to_string()
    }

    fn datadir() -> String {
        basedir() + "/../akaza-data/data/"
    }

    fn load_unigram() -> anyhow::Result<MarisaSystemUnigramLM> {
        let datadir = datadir();
        let path = datadir + "/unigram.model";
        MarisaSystemUnigramLM::load(&path)
    }

    fn load_bigram() -> MarisaSystemBigramLM {
        let datadir = datadir();
        let path = datadir + "/bigram.model";

        MarisaSystemBigramLM::load(&path).unwrap()
    }

    #[test]
    fn test_load() -> anyhow::Result<()> {
        let unigram: MarisaSystemUnigramLM = load_unigram()?;
        let bigram = load_bigram();

        let (id1, score1) = unigram.find("私/わたし").unwrap();
        assert!(id1 > 0);
        assert!(score1 > 0.0_f32);

        let (id2, score2) = unigram.find("と/と").unwrap();
        assert!(id2 > 0);
        assert!(score2 > 0.0_f32);

        println!("id1={}, id2={}", id1, id2);

        let bigram_score = bigram.get_edge_cost(id1, id2).with_context(|| {
            format!(
                "bigram.num_entries={} id1={} id2={}",
                bigram.num_keys(),
                id1,
                id2
            )
        })?;
        assert!(bigram_score > 0.0_f32);

        println!("BigramScore={}", bigram_score);
        Ok(())
    }
}
