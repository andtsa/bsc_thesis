# Quantifying Uncertainty due to Ties in Rank Correlation Coefficients
*An algorithmic approach to computing the bounds of uncertainty.*

## Abstract

Rank correlation coefficients are a common tool for describing similarity between ordered data. 
This study examines the use of the popular coefficient Kendall’s 𝜏, 
specifically in the case where the rankings contain tied items that should not be tied. 
Ties in this case represent uncertainty in the ranking,
induced by the system that produced it,
usually due to effects such as missing information or loss of precision (rounding).
We propose two variants, 𝜏𝑚𝑖𝑛 and 𝜏 𝑚𝑎𝑥 , 
which represent the lowest and highest possible correlation over all ways of arbitrating tied items.
Our contribution is a novel quadratic-time algorithm for computing an arbitration of ties which yields the extremal correlation values 𝜏 𝑚𝑖𝑛 , 𝜏 𝑚𝑎𝑥.
We formally prove the correctness of the algorithm for the original Kendall’s 𝜏,
and we suggest an adaptation for weighted variants of 𝜏, 
such as 𝜏𝐴𝑃 by Yilmaz et al. and 𝜏h by Vigna.
Empirical evaluation on both synthetic ranking pairs and TREC ad-hoc system outputs
demonstrates that ties often induce wide intervals [𝜏𝑚𝑖𝑛 , 𝜏𝑚𝑎𝑥],
indicating that no single value can fully encapsulate the uncertainty in correlation.
These wide intervals also appear in rankings where current methods of computing 𝜏 correlation in presence of ties,
namely 𝜏𝑎 and 𝜏𝑏, have values large enough (≥ 0.9) for researchers to use as evidence of strong correlation. 
This indicates that currently used methods may yield false positive results. 
By reporting 𝜏 alongside its uncertainty bounds 𝜏 𝑚𝑖𝑛 and 𝜏 𝑚𝑎𝑥 , 
researchers are able to make more informed decisions,
by demonstrating the reliability of correlation in presence of uncertainty-induced ties.


## Link to paper
[https://repository.tudelft.nl](https://repository.tudelft.nl) (pending)

# Source code organisation

- `lib/`: shared definitions
    - tau correlation
    - rankings (with and without ties)
- `solver/`: the two solvers, 
    - the one of the proposed graph-based algorithm,
    - a naîve brute force solver
- `verifier/`: compare the outputs of a solver to a solutions file
    - `comp`: utility to compare two solvers given a csv of rankings
- `eval/`: process outputs of 
- `plots/`: R code for generating the plots used in the paper.

