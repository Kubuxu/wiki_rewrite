extern crate wiki_rewrite;

fn main() {
    //wiki_rewrite::rewrite("/storage/home/achin/devel/rust_libs/zim/zim_output_3/A");
    wiki_rewrite::rename("/storage/home/achin/devel/rust_libs/zim/zim_output_3/A", 2);
    wiki_rewrite::rename("/storage/home/achin/devel/rust_libs/zim/zim_output_3/I/m", 1);
}
