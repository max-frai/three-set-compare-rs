About
----
Rust implementation of three set compare algorithm, which is steady for words ordering and spelling mistakes.
It's best usage case is comparing news titles, for example. Don't use it for data longer than 255 characters.

The main idea is to find characters which belongs to set of letters both of words.

    A = {a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z}
    S1 = {s1, s2, s3 ... sn}
    S2 = {s1, s2, s3 ... sm}

    S1 ⊆ A, S2 ⊆ A

    Sim(S1, S2) = ||S1 ∩ A| - |S2 ∩ A||/|S1| + |S2|

It's not optimized for now, but should be very cheap in future.

Benchmark
---

MacBook Pro (15-inch, 2017), 2.9 GHz Intel Core i7

    test tests::bench_similarity ... bench: 30,278 ns/iter (+/- 2,312)

Usage
--
    use three_set_compare::ThreeSetCompare;
    let comparator = ThreeSetCompare::new();

    // 1.0
    let similarity = comparator.similarity("First phrase here!", "Here firts phrase");

License
---
MIT