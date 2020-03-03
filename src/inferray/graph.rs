use sophia::graph::inmem::*;
use sophia::graph::GTripleSource;
use sophia::graph::Graph;
use sophia::term::{Term, TermData};
use sophia::triple::streaming_mode::{ByTermRefs, StreamedTriple};
use sophia::triple::{stream::TripleSource, Triple};

use std::convert::Infallible;

use super::NodeDictionary;
use super::TripleStore;

pub struct InfGraph {
    pub dictionary: NodeDictionary,
}

impl Graph for InfGraph {
    type Triple = ByTermRefs<std::rc::Rc<str>>;
    type Error = Infallible;

    fn triples(&self) -> GTripleSource<Self> {
        let mut v: Vec<Result<StreamedTriple<ByTermRefs<std::rc::Rc<str>>>, Self::Error>> =
            Vec::new();
        // eprintln!(":{:?}", self.dictionary.ts.elem);
        for (ip, chunk) in (&self.dictionary.ts.elem).iter().enumerate() {
            // eprintln!("p: {}", p);
            if !chunk[0].is_empty() {
                let ip = NodeDictionary::idx_to_prop_idx(ip);
                let p = self.dictionary.get_term(ip);
                for pair in &chunk[0] {
                    // eprintln!("pso: {} {:?}", ip, pair);
                    let s = self.dictionary.get_term(pair[0]);
                    let o = self.dictionary.get_term(pair[1]);
                    v.push(Ok(StreamedTriple::by_term_refs(s, p, o)));
                }
            }
        }
        Box::from(v.into_iter())
    }

    //     fn triples_with_s<'s, T>(&'s self, s: &'s Term<T>) -> GTripleSource<'s, Self>
    //     where
    //         T: TermData,
    //     {
    //         Box::new(self.triples().filter_ok(move |t| t.s() == s))
    //     }

    fn triples_with_p<'s, T>(&'s self, p: &'s Term<T>) -> GTripleSource<'s, Self>
    where
        T: TermData,
    {
        println!("My impl");
        let mut v: Vec<Result<StreamedTriple<ByTermRefs<std::rc::Rc<str>>>, Self::Error>> =
            Vec::new();
        if let Some(ip) = self.dictionary.get_index(p) {
            let idx = NodeDictionary::prop_idx_to_idx(ip);
            let chunk = &self.dictionary.ts.elem[idx];
            if !chunk[0].is_empty() {
                let p = self.dictionary.get_term(ip);
                for pair in &chunk[0] {
                    let s = self.dictionary.get_term(pair[0]);
                    let o = self.dictionary.get_term(pair[1]);
                    v.push(Ok(StreamedTriple::by_term_refs(s, p, o)));
                }
            }
        }
        Box::from(v.into_iter())
    }

    //     fn triples_with_o<'s, T>(&'s self, o: &'s Term<T>) -> GTripleSource<'s, Self>
    //     where
    //         T: TermData,
    //     {
    //         Box::new(self.triples().filter_ok(move |t| t.o() == o))
    //     }

    //     fn triples_with_sp<'s, T, U>(
    //         &'s self,
    //         s: &'s Term<T>,
    //         p: &'s Term<U>,
    //     ) -> GTripleSource<'s, Self>
    //     where
    //         T: TermData,
    //         U: TermData,
    //     {
    //         Box::new(self.triples_with_s(s).filter_ok(move |t| t.p() == p))
    //     }

    //     fn triples_with_so<'s, T, U>(
    //         &'s self,
    //         s: &'s Term<T>,
    //         o: &'s Term<U>,
    //     ) -> GTripleSource<'s, Self>
    //     where
    //         T: TermData,
    //         U: TermData,
    //     {
    //         Box::new(self.triples_with_s(s).filter_ok(move |t| t.o() == o))
    //     }

    //     fn triples_with_po<'s, T, U>(
    //         &'s self,
    //         p: &'s Term<T>,
    //         o: &'s Term<U>,
    //     ) -> GTripleSource<'s, Self>
    //     where
    //         T: TermData,
    //         U: TermData,
    //     {
    //         Box::new(self.triples_with_p(p).filter_ok(move |t| t.o() == o))
    //     }

    //     fn triples_with_spo<'s, T, U, V>(
    //         &'s self,
    //         s: &'s Term<T>,
    //         p: &'s Term<U>,
    //         o: &'s Term<V>,
    //     ) -> GTripleSource<'s, Self>
    //     where
    //         T: TermData,
    //         U: TermData,
    //         V: TermData,
    //     {
    //         Box::new(self.triples_with_sp(s, p).filter_ok(move |t| t.o() == o))
    //     }
}

impl InfGraph {
    fn encode_triple<TD>(&mut self, t: &dyn Triple<TermData = TD>) -> [i64; 3]
    where
        TD: std::convert::AsRef<str> + std::clone::Clone + std::cmp::Eq + std::hash::Hash,
    {
        let contains_prop = contains_prop_in_s_or_o(t);
        let mut s: i64 = -1;
        let mut o: i64 = -1;
        let mut p: i32 = -1;
        let s_str = t.s().value();
        let o_str = t.o().value();
        let p_str = t.p().value();
        // Property will always be property
        p = self.dictionary.add_property(&p_str);
        if contains_prop != -1 {
            match contains_prop {
                1 => {
                    s = self.dictionary.add_property(&s_str).into();
                    o = self.dictionary.add(&o_str);
                }
                3 => {
                    s = self.dictionary.add_property(&s_str).into();
                    o = self.dictionary.add_property(&o_str).into();
                }
                _ => (),
            }
        } else {
            // Add a regular triple
            s = self.dictionary.add(&s_str);
            o = self.dictionary.add(&o_str);
        }
        [s, p as i64, o]
    }

    pub fn size(&mut self) -> usize {
        self.dictionary.ts.size()
    }
}

impl<TS> From<TS> for InfGraph
where
    TS: TripleSource,
{
    fn from(mut ts: TS) -> Self {
        let store = TripleStore::new();
        let dictionary = NodeDictionary::new(store);
        let mut me = Self { dictionary };
        ts.for_each_triple(|t| {
            let rep = me.encode_triple(&t);
            //eprintln!("{:?}", rep);
            me.dictionary.ts.add_triple(rep);
        })
        .expect("Streaming error");
        me.dictionary.ts.sort();
        me
    }
}

// Should return -1 if both s and o are res,
// 1 if s is prop and o is res,
// and 3 if both s and o are prop
fn contains_prop_in_s_or_o<TD>(t: &dyn Triple<TermData = TD>) -> i32 {
    -1
}
