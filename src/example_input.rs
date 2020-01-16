
pub static EXAMPLE: &str = r####"
// 1 whitespace
%: 1wh { }
%: 1wh {
}
%: newline {
}
// 0 whitespace
%: 0wh {}
%: nwh {}
// n whitespace
%: nwh {::1wh::nwh}

%: az {a} %: az {b} %: az {c} %: az {d} %: az {e} %: az {f} %: az {g} %: az {h} %: az {i} %: az {j} %: az {k} %: az {l} %: az {m} %: az {n} %: az {o} %: az {p} %: az {q} %: az {r} %: az {s} %: az {t} %: az {u} %: az {v} %: az {w} %: az {x} %: az {y} %: az {z}
%: AZ {A} %: AZ {B} %: AZ {C} %: AZ {D} %: AZ {E} %: AZ {F} %: AZ {G} %: AZ {H} %: AZ {I} %: AZ {J} %: AZ {K} %: AZ {L} %: AZ {M} %: AZ {N} %: AZ {O} %: AZ {P} %: AZ {Q} %: AZ {R} %: AZ {S} %: AZ {T} %: AZ {U} %: AZ {V} %: AZ {W} %: AZ {X} %: AZ {Y} %: AZ {Z}
%: aZ {::az}
%: aZ {::AZ}
%: 09 {0} %: 09 {1} %: 09 {2} %: 09 {3} %: 09 {4} %: 09 {5} %: 09 {6} %: 09 {7} %: 09 {8} %: 09 {9}

// non white space special characters
%: specialchar {.!}%: specialchar {."}%: specialchar {.#}%: specialchar {.$}%: specialchar {.%}%: specialchar {.&}%: specialchar {.'}%: specialchar {.(}%: specialchar {.)}%: specialchar {.*}%: specialchar {.+}%: specialchar {.,}%: specialchar {.-}%: specialchar {..}%: specialchar {./}%: specialchar {.:}%: specialchar {.;}%: specialchar {.<}%: specialchar {.=}%: specialchar {.>}%: specialchar {.?}%: specialchar {.@}%: specialchar {.[}%: specialchar {.\}%: specialchar {.]}%: specialchar {.^}%: specialchar {._}%: specialchar {.`}%: specialchar {.{}%: specialchar {.|}%: specialchar {.}}%: specialchar {.~}

%: alnum {::aZ}
%: alnum {::09}
%: alnum_ {_}
%: alnum_ {::alnum}
// non white space characters
%: opaquechar {::specialchar}
%: opaquechar {::alnum}
%: anychar {::1wh}
%: anychar {::opaquechar}

%: ident {::alnum_}
%: ident {::alnum_::ident}

// generators are rules that turn a %!: rule into a % : rule
// %!: rules can have cooler syntax

// generate rule that matches a string until `end` (if it has not been escaped with `esc`)
// does not escape matched string.
// usage: %!: `ruleName` = end with `end`, escape `esc`
// this generator uses the namespace `ruleName`_***
%/: genRuleEndText {%!.:::nwh:ruleName:ident::nwh.=::nwh.end::nwh.with::nwh:end:anychar::nwh.,::nwh.escape::nwh:esc:anychar} {
    %.: :ruleName {.:x.:anychar.:r.::ruleName.} {.:x.:r.}
    %.: :ruleName {..:esc..:end.:r.::ruleName.} {..:esc..:end.:r.}
    %.: :ruleName {..:end.} {.}
}

// generate rule that matches a string until `end` (if it has not been escaped with `esc`)
// does escape matched string.
// usage: %!: `ruleName` = any text, end with `end`, escape with `esc`
// this generator uses the namespace `ruleName`_***
%: genRuleEndText {%!.:::nwh:ruleName:ident::nwh.=::nwh.any::nwh.text::nwh.,::nwh.end::nwh.with::nwh:end:anychar::nwh.,::nwh.escape::nwh.with::nwh:esc:anychar} {
    %.: :ruleName {.:x.:anychar.:r.::ruleName.} {.:x.:r.}
    %.: :ruleName {..:esc.:c.:anychar.:r.::ruleName.} {.:c.:r.}
    %.: :ruleName {..:end.} {.}
}

// idea how to implement namespaces with static contracts:
// a rule that wants to be the only one to use the namespace abc_**
// can formulate a static contract requiring no identifier to have this form.

// repeat innerRule 0 or more times
// %!: `ruleName` = `innerRule`*
// only uses namespace `ruleName`
%: genRuleStar {%!.:::nwh:ruleName:ident::nwh.=::nwh:innerRule:ident::nwh.*} {
    %: :ruleName {.} {.}
    %: :ruleName {.:x.::innerRule.:y.::ruleName.} {.:x.:y.}
}

// repeat innerRule 1 or more times
// %!: `ruleName` = `innerRule`+
// only uses namespace `ruleName`
%: genRulePlus {%!.:::nwh:ruleName:ident::nwh.=::nwh:innerRule:ident::nwh.+} {
    %: :ruleName {.:x.::innerRule.} {.:x.}
    %: :ruleName {.:x.::innerRule.:y.::ruleName.} {.:x.:y.}
}

// repeat innerRule 1 or more times
// %!: `ruleName` = `innerRule`?
// only uses namespace `ruleName`
%: genRuleMaybe {%!.:::nwh:ruleName:ident::nwh.=::nwh:innerRule:ident::nwh.?} {
    %: :ruleName {.} {.}
    %: :ruleName {.:x.::innerRule.} {.:x.}
}

// now activate the generators.
// for that, we activate a single generator, which is used to activate other generators:
//     %!: apply anywhere `genName`
// which applies the given rule anywhere in the following text wherever it can match.
// this generator uses the namespace `genName`_**
%: genApply {%!.:::nwh.apply::nwh.anywhere::nwh:ruleName:ident} {
    %.: :ruleName._1generable {.:x.:anychar.} {.:x.}
    %.: :ruleName._1generable {.:r.::ruleName.} {.:r.}
    %.: :ruleName._generable {.}
    %.: :ruleName._generable {.:r.::ruleName._1generable.:g.::ruleName._generable.} {.:r.:g.}
    %.: {.:g.::ruleName._generable.} {.:g.}
}

// %!: apply anywhere genApply
%: 1generable {:x:anychar}
%: 1generable {:r:genApply}
%: generable {}
%: generable {:r:1generable:g:generable}
%: {:g:generable}

// infinite loop occurs at this chunk of text
%!: apply anywhere genRuleEndText
%!: apply anywhere genRuleMaybe
%!: apply anywhere genRulePlus
%!: apply anywhere genRuleStar

// function syntax: f(any text that does not contain unescaped .))
// function input will be unescaped content between '(' and ')'
// function call is replaced with function output
%/: f_input = any text, end with ), escape with .
%/: f_escape {escape.(:x:f_input} {:c}

%/: {.;::nwh:text:braced_esctext} {:text};

%!: manynumbers=09*


%: {:t:manynumbers} {success. .{:t.}}
"####;
