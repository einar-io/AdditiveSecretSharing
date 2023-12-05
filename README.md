# Additive Secret Sharing

This is a proof-of-concept implementation the Additive Secret Sharing scheme
explained by professor Ivan Damgård here[^1].  Additive Secret Sharing is a
simple example of a Multi Party Computation.

To try it out:

    git clone https://github.com/einar-io/AdditiveSecretSharing.git
    RUST_LOG=info cargo run


##### Caveats

There are no attempts done at error handling, automated testing or efficiency.


[^1]: [Multi-Party Computation simplified: Ivan Damgård, Co-founder/Chief
Cryptographer-Partisia Blockchain](https://www.youtube.com/watch?v=vRVudJADQLk)
