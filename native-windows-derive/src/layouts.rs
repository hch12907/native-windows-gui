use crate::shared::Parameters;


#[derive(Clone, Copy)]
pub struct GridLayoutChild {
    pub col: u32,
    pub row: u32,
    pub col_span: u32,
    pub row_span: u32
}

#[derive(Clone, Copy)]
pub struct FlexboxLayoutChild {
}


pub enum LayoutChild {
    Init { field_name: String, params: Parameters },
    Grid(GridLayoutChild),
    Flexbox(FlexboxLayoutChild),
}

impl LayoutChild {

    pub fn prepare(field: &syn::Field) -> Option<LayoutChild> {
        let field_name = field.ident.as_ref().map(|i| i.to_string())
            .unwrap_or("Unnamed".to_string());

        let map_attr = |attr: &syn::Attribute| {
            LayoutChild::Init {
                field_name,
                params: syn::parse2(attr.tokens.clone()).unwrap()
            }
        };
        
        field.attrs.iter()
            .find(|attr| attr.path.get_ident().map(|id| id == "nwg_layout_item").unwrap_or(false) )
            .map(map_attr)
    }

    pub fn parse(&mut self, parent_type: &syn::Ident) {
        let int_value = |expr: &syn::Expr| -> u32 {
            match expr {
                syn::Expr::Lit(lit) => 
                    match &lit.lit {
                        syn::Lit::Int(i) => { i.base10_parse().unwrap() },
                        _ => panic!("Layout item members must be int literal.")
                    },
                _ => panic!("Layout item members must be int literal.")
            }
        };

        let [mut col, mut row, mut col_span, mut row_span] = [0, 0, 1, 1];

        match self {
            LayoutChild::Init{ params: p, .. } => for p in p.params.iter() {
                let attr_name = p.ident.to_string();
                match &attr_name as &str {
                    // Grid layout
                    "col" => { col = int_value(&p.e) },
                    "row" => { row = int_value(&p.e) },
                    "col_span" => { col_span = int_value(&p.e) },
                    "row_span" => { row_span = int_value(&p.e) },
                    _ => {}
                }
            },
            _ => panic!("Called parse on a non-Init child layout")
        };

        if parent_type == "GridLayout" {
            *self = LayoutChild::Grid( GridLayoutChild { col, col_span, row, row_span } );
        } else if parent_type == "FlexboxLayout" {
            *self = LayoutChild::Flexbox( FlexboxLayoutChild { } );
        } else {
            panic!("Unknown parent type: {:?}", parent_type);
        }
    }

    pub fn parent_matches(&self, parent: &syn::Ident) -> bool {
        match self {
            LayoutChild::Init{ params: p, .. } => p.params
                .iter()
                .filter(|p| p.ident == "layout")
                .any(|p| match &p.e {
                    syn::Expr::Path(exp_path) => 
                        exp_path.path.segments.last()
                            .map(|seg| &seg.ident == parent)
                            .unwrap_or(false),
                    _ => false
                } ),
            _ => panic!("parent_matches called on non-init values")
        }
    }

}

//
// Main layout
//

pub fn layout_parameters(field: &syn::Field) -> (Vec<syn::Ident>, Vec<syn::Expr>) {
    let member = match field.ident.as_ref() {
        Some(m) => m,
        None => unreachable!()
    };

    let nwg_layout = |attr: &&syn::Attribute| {
        attr.path.get_ident()
          .map(|id| id == "nwg_layout" )
          .unwrap_or(false)
    };

    let attr = match field.attrs.iter().find(nwg_layout) {
        Some(attr) => attr,
        None => unreachable!()
    };

    let layout: Parameters = match syn::parse2(attr.tokens.clone()) {
        Ok(a) => a,
        Err(e) => panic!("Failed to parse field #{}: {}", member, e)
    };

    let params = layout.params;
    let mut names = Vec::with_capacity(params.len());
    let mut exprs = Vec::with_capacity(params.len());

    for p in params {
        names.push(p.ident);
        exprs.push(p.e);
    }

    (names, exprs)
}
