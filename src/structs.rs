use std::{fmt::Debug, ops};

#[derive(Debug, Clone)]
pub enum Rarity {
    Covert,
    Classified,
    Restricted,
    Rare,
    MilSpec,
    None,
}

impl Rarity {
    pub fn convert(rarity: Option<String>) -> Rarity {
        let mut lower_rarity = match rarity {
            Some(x) => x,
            None => return Rarity::None
        };
        lower_rarity = lower_rarity.to_ascii_lowercase();
    
        if lower_rarity.contains("covert") {
            Rarity::Covert
        } else if lower_rarity.contains("classified") {
            Rarity::Classified
        } else if lower_rarity.contains("restricted") {
            Rarity::Restricted
        } else if lower_rarity.contains("rare") {
            Rarity::Rare
        } else if lower_rarity.contains("mil-spec") {
            Rarity::MilSpec
        } else {
            Rarity::None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CaseElement {
    pub url: Option<String>,
    pub image: Option<String>,
    pub name: Option<String>,
    pub price: Option<String>,
    pub items: Option<Vec<Items>>,
    pub knifes: Option<Box<CaseElement>>,
}

unsafe impl Send for CaseElement {}
unsafe impl Send for Items {}
unsafe impl Send for Rarity {}

impl CaseElement {
    pub fn new(url: Option<String>, image: Option<String>, name: Option<String>, price: Option<String>) -> Self {
        Self {
            url,
            image,
            name,
            price,
            items: Some(Vec::new()),
            knifes : None
        }
    }

    #[allow(dead_code)]
    pub fn print_case_elements(case_elements: Vec<CaseElement>) {
        for i in case_elements {
            println!("{:#?}", i);
        }
    }
}

impl ops::Add for CaseElement {
    type Output = CaseElement;

    fn add(self, rhs: Self) -> Self::Output {
        let mut temp = CaseElement::new(self.url, self.image, self.name, self.price);
        temp.knifes = Some(Box::new(rhs));
        temp.items = self.items;
        return temp;
    }
}

#[derive(Debug, Clone)]
pub struct Items {
    pub name: Option<String>,
    pub rarity: Option<Rarity>,
    pub nonstatprice: Option<String>,
    pub statprice: Option<String>,
    pub image: Option<String>,
}

impl Items {
    pub fn new(
        name: Option<String>,
        rarity: Option<Rarity>,
        nonstatprice: Option<String>,
        statprice: Option<String>,
        image: Option<String>,
    ) -> Self {
        Self {
            name,
            rarity,
            nonstatprice,
            statprice,
            image
        }
    }
}