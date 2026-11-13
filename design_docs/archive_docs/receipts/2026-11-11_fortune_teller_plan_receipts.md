# Receipts: fortune teller plan

## 2026-11-13

### misaki lexicons

- Inputs: `misaki-us_gold.json` (3,000,469 bytes) and
  `misaki-us_silver.json` (3,099,517 bytes) as embedded in the `loqui-g2p`
  0.4.0 crate source, which its NOTICE says are misaki 0.9.4 unmodified.
  Keys lowercased; an entry with part-of-speech variants uses its default.
- Loaded: gold 89,359, silver 93,361, in both 4,321.
- Gold: 36,916 words in CMUdict, 52,443 not (17,953 of those in
  LibriSpeech). Against CMUdict 77.9% exact, 4.0% phone error; against
  Wiktionary US on CMUdict words (26,784) 79.8%, 3.7%; on non-CMUdict words
  (13,882) 68.4%, 5.6%; against LibriSpeech on non-CMUdict words 53.7%,
  9.4%. Primary stress on the same vowel as some CMUdict variant: 35,314 of
  36,771 (96.0%).
- Silver: 14,303 in CMUdict, 79,058 not (11,372 in LibriSpeech). Against
  CMUdict 85.8%, 2.8%; Wiktionary on CMUdict words (5,282) 88.9%, 2.3%; on
  non-CMUdict words (4,280) 73.3%, 5.4%; against LibriSpeech 69.6%, 6.3%.
  Primary stress: 13,879 of 14,302 (97.0%).
- WordNet single words 68,869, not in CMUdict 36,701: in gold 21,134
  (57.6%); gold or silver 24,049 (65.5%). CMUdict + gold 53,302 (77.4%);
  + silver 56,217 (81.6%); + LibriSpeech 57,934 (84.1%).

### Guesser trial

- Harness: a scratch Cargo project at `Code/testing/soothsayer/g2p-trial/`
  (source, `words.txt` and `out.tsv` kept; build output removed) depending
  on `grapheme_to_phoneme` 0.1.0 and `loqui-g2p` 0.4.0 from crates.io.
  Neither has a build script. Release build; 45 s to run both over the list.
- Word list: LibriSpeech-only words also in Wiktionary US, letters only:
  11,036. Scored as in the yardstick below; loqui's phonemes mapped from its
  misaki symbols first. 11,029 words scored.
- All words: LibriSpeech 52.0% exact, 11.7% phone error;
  grapheme_to_phoneme 43.7%, 18.2%; loqui-g2p 65.2%, 7.8%.
- 7,840 of the words are keys in loqui's `misaki-us_gold.json` or
  `misaki-us_silver.json`. On the 3,194 that are not: LibriSpeech 44.7%,
  15.7%; grapheme_to_phoneme 39.7%, 20.6%; loqui-g2p 48.0%, 15.1%.
- LibriSpeech and grapheme_to_phoneme agree on 5,039 words: 75.1% exact,
  5.8% phone error there. On the 5,993 where they differ: LibriSpeech
  32.6%, 15.6%; grapheme_to_phoneme 17.3%, 26.4%.

### LibriSpeech against CMUdict

Words lowercased, alternates folded. cmudict 125,929 words; librispeech
200,000. In both 66,016: at least one identical pronunciation 64,005 (97.0%);
every librispeech pronunciation present in cmudict 63,498 (96.2%). Only in
librispeech 133,984 (3.7 MB as text); only in cmudict 59,913.

### Wiktionary yardstick

- Inputs: `eng_latn_us_broad.tsv` (2,875,614 bytes, 106,931 lines, SHA-256
  `f304b8f0565466e8abfe4c04a30d99421e1590d2aa17c4a08ebfa5942c587e75`) and
  `eng_latn_uk_broad.tsv` (2,915,740 bytes, 106,688 lines, SHA-256
  `f8ca476aa2e96cb22cafbf9099d689b5902a9d321579055081909b2eeafa1b8d`) from
  `CUNY-CL/wikipron` at `d282e848a2`. Kept at
  `Code/testing/soothsayer/wikipron-d282e848/`.
- Method: IPA mapped to stressless ARPAbet by a one-off table (218 US and 250
  UK lines had a symbol outside it and were skipped); ER written as AH R on
  both sides; edit distance per word, best pair of listed variants; phone
  error is total edits over total reference phones. The script was not kept.
- Loaded: Wiktionary US 79,220 words, UK 79,801.
- Against Wiktionary US: CMUdict words 34,963, exact 82.1%, phone error
  3.6%; LibriSpeech words also in CMUdict 30,721, 84.1%, 3.2%;
  LibriSpeech-only 11,130, 51.9%, 11.8%; the WordNet words among those
  5,220, 53.3%, 10.0%.
- Coverage of WordNet single words (68,869): not in CMUdict 36,701, of which
  Wiktionary US 9,972, US or UK 11,248; in neither CMUdict nor LibriSpeech
  23,126, of which US 4,752, US or UK 5,358. CMUdict + Wiktionary US 61.2%;
  + UK 63.0%; + LibriSpeech 74.2%.

## 2026-11-11

### Turquet wasm compile check

`cargo check --lib --target wasm32-unknown-unknown --locked` in
`crates/turquet`, debug info off: finished clean in 1m 19s with six lint
warnings. The build output it left was removed afterwards.

### WordNet size and CMUdict coverage

- Input: `english-wordnet-2025.xml.gz` from the `2025-edition` release of
  `globalwordnet/english-wordnet`, 11,363,503 bytes, SHA-256
  `9ca6d1dcb75f822fdd66617f7d9da48142ace38dd544d6ad5e2feca1674ad3fe`;
  89,237,271 bytes unpacked. Kept at `Code/testing/soothsayer/oewn-2025/`.
- Pronunciations: `knot-editor/crates/mora-cmudict/data/cmudict.dict`,
  125,929 distinct words after folding alternates.
- Method: a one-off line scanner over the XML, lemmas lowercased; a single
  word is covered if it is in the dictionary, a phrase if every alphabetic
  part is. The script was not kept.
- Output:
  - lemmas 127,311; single plain words 68,869, covered 32,168 (46.7%);
    phrases and other forms 58,442, covered 43,834 (75.0%); overall 76,002
    (59.7%).
  - synsets 107,519; synset relations 234,810; sense relations 120,254.
  - integer-coded graph JSON 5.8 MB raw, 1.6 MB gzip; pronunciations needed
    34,459 words, 1.0 MB raw, 0.3 MB gzip.
### Second pronunciation sources

- `english_us_arpa.dict` from the `dictionary-english_us_arpa-v3.0.0` release
  of `MontrealCorpusTools/mfa-models`, 7,186,498 bytes, 208,651 lines,
  SHA-256 `e8c6c7b036ae2b7c78d2768b8dc6b1f9359175b842956d00b48c53c9c332e6b0`;
  199,860 distinct words. Removed after D33 chose the other source.
- `librispeech-lexicon.txt` from `openslr.org/resources/11/`, 5,627,653
  bytes, 206,508 lines, SHA-256
  `d722bc29908cd338ae738edd70f61826a6fca29aaa704a9493f0006773f79d71`; 200,000
  distinct words. Kept at `Code/testing/soothsayer/librispeech-lexicon/`.
- Same method as the coverage run above. Single words covered, of 68,869:
  cmudict 32,168; mfa 41,307; librispeech 41,366; cmudict+mfa 45,715;
  cmudict+librispeech 45,743; all three 45,743. Phrases covered, of 58,442:
  43,834; 46,105; 46,210; 47,901; 47,916; 47,916.
- Controls: "cat" and "soothsayer" present in all three; "gargle",
  "expository", "conjunct" absent from cmudict and present in the other two.
- 23,126 single words in none of the three.

- Control for the first run: `grep` on the dictionary file confirmed "gargle", "expository" and
  "conjunct" are absent, while "conjunction" and "insert" are present.
