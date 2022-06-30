use chrono::{Local};
use regex::Regex;
use std::fs;

struct LineItem {
    po1_segment: String,
    ack_segment: String,
}

struct Order {
    details: Vec<String>,
    line_items: Vec<LineItem>,
}

struct Acknowledgement {
    isa_segment: String,
    gs_segment: String,
    st_segment: String,
    bak_segment: String,
    n1_sf_segment: String,
    n1_st_segment: String,
    n3_st_segment: String,
    n4_st_segment: String,
    order: Order,
    control_tt_segment: String,
    se_segment: String,
    iea_segment: String,
}

// ST*855*68800002<0x85>
// BAK*06*AD*4515216839*20220627****525876*20220627<0x85>
// N1*ST*Medline Industries LP*92*C54<0x85>
// N1*SF*Busse Hospital Disposables*92*2750<0x85>
// PO1*10*30*CA*130**VC*4247*IN*BHD4247<0x85>
// ACK*AC*30*CS*002*20220627<0x85>
// CTT*1*60<0x85>
// SE*8*68800002<0x85>
// ST*855*68800003<0x85>
// BAK*06*AC*4515211735*20220624****525838*20220624<0x85>
// N1*ST*Medline Industries LP*92*B31<0x85>
// N1*SF*Busse Hospital Disposables*92*2750<0x85>
// PO1*10*1*CA*75.85**VC*282*IN*BHD282<0x85>
// ACK*AC*1*CS*002*20220624<0x85>
// CTT*1*2<0x85>
// SE*8*68800003<0x85>

// ISA*00*          *00*          *01*002418234T     *01*012430880      *220628*0940*U*00201*000005452*1*P*}
// GS*IN*002418234T*HS810*220628*0940*5452*X*003040

// IEA*1*000005110
fn remove_special_instructions(order: String) -> String {
    let re = Regex::new("^\"S(R|I)\"~\".*\"").unwrap();
    let order_lines = order.lines();
    let mut order_lines_clean = Vec::new();

    for line in order_lines {
        match re.find(line) {
            Some(_) => {
                // do nothing
            }
            None => {
                let line_clean = line.replace("\"", "");
                order_lines_clean.push(line_clean);
            }
        }
    }
    order_lines_clean.join("\n")
}

fn increment_control_number() -> Vec<String> {
    let text = fs::read_to_string("./control_number.txt").expect("error reading file");
    let date_string = text.split(";").collect::<Vec<&str>>()[0];

    match Local::now().format("%y%m%d").to_string() == date_string {
        true => {
            println!("Today's date is the same as the last date in the control number file");
            let control_number = text.split(";").collect::<Vec<&str>>()[1];
            let control_number_int = control_number.parse::<i32>().unwrap();
            let control_number_next = control_number_int + 1;
            let control_number_str = format!("{};{}", date_string, control_number_next.to_string());
            fs::write("control_number.txt", control_number_str).unwrap();
            return vec![date_string.to_string(), control_number_next.to_string()];
        }
        false => {
            println!("Today's date is different than the last date in the control number file");
            let control_number_str = format!("{};{}", Local::now().format("%y%m%d").to_string(), 1);

            fs::write("control_number.txt", control_number_str).unwrap();
            return vec![Local::now().format("%y%m%d").to_string(), 1.to_string()];
        }
    }
}

fn get_trading_partner_id(partner: String) -> String {
    let mut partners: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    partners.insert("HENRYSCHEIN".to_string(), "012430880      ".to_owned());
    partners.insert("BUSSE".to_string(), "002418234T     ".to_owned());
    
    match partners.get(&partner) {
        Some(id) => {
            return id.to_string();
        }
        None => {
            println!("Partner not found");
            return "".to_string();
        }
    }
}

fn main() {
    let filename: String = "/mnt/c/temp/henryschein_edi.txt".to_owned();

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    let mut orders: Vec<String> = Vec::new();

    for line in contents.split("\"H\"") {
        let clean_line = remove_special_instructions("\"H\"".to_owned() + line);

        orders.push(clean_line);
    }

    // print each order in orders
    for order in &orders[1..] {
        println!("{}", order);
    }

    let mut acks: Vec<Acknowledgement> = Vec::new();

    for order in &orders[1..] {
        let isa_control_number_raw = increment_control_number();
        let isa_control_number = format!(
            "{}{:0>3}",
            isa_control_number_raw[0], isa_control_number_raw[1]
        );
        let control_number_raw = increment_control_number();
        let control_number = format!("{}{:0>3}", control_number_raw[0], control_number_raw[1]);
        

        // get date %y%m%d get time %H%M

        let date_time = Local::now();
        let date = date_time.format("%y%m%d").to_string();
        let time = date_time.format("%H%M").to_string();

        let mut ack: Acknowledgement = Acknowledgement {
            isa_segment: format!("ISA*00*          *00*          *01*{}*01*{}*{}*{}*U*00201*{}*1*P*", &get_trading_partner_id("BUSSE".to_owned()),&get_trading_partner_id("HENRYSCHEIN".to_owned()), date, time, isa_control_number),
            gs_segment: format!("GS*IN*{}*HS855*{}*{}*5452*X*003040", &get_trading_partner_id("BUSSE".to_owned()), date, time),
            st_segment: format!("ST*855*{}", control_number),
            bak_segment: "".to_owned(),
            n1_sf_segment: "".to_owned(),
            n1_st_segment: "".to_owned(),
            n3_st_segment: "".to_owned(),
            n4_st_segment: "".to_owned(),
            order: Order {
                details: Vec::new(),
                line_items: Vec::new(),
            },
            control_tt_segment: "".to_owned(),
            se_segment: format!("SE*855*{}", control_number),
            iea_segment: format!("IEA*{}", isa_control_number),
        };

        for line in order.lines() {
            let split_line = line.split("~").collect::<Vec<&str>>();

            match split_line[0] {
                "H" => {
                    let vendor_so = split_line[8];
                    let received_date = split_line[9];
                    let customer_po = split_line[10];

                    ack.bak_segment = format!(
                        "BAK*00*AD*{}*{}****{}*{}",
                        customer_po, received_date, vendor_so, received_date
                    );

                    let sf_name = split_line[50];

                    ack.n1_sf_segment = format!("N1*SF*{}**", sf_name);

                    let st_name = split_line[43].replace("//", " ");
                    let st_identifier = split_line[62];
                    let st_address = split_line[44];
                    let st_address_2 = split_line[45];
                    let st_city = split_line[46];
                    let st_state = split_line[47];
                    let st_zip = split_line[48];

                    ack.n1_st_segment = format!("N1*ST*{}*92*{}", st_name, st_identifier);
                    ack.n3_st_segment = format!("N3*{}*{}", st_address, st_address_2);
                    ack.n4_st_segment = format!("N4*{}*{}*{}*", st_city, st_state, st_zip);
                }
                "D" => {
                    ack.order.details.push(split_line[1].to_string());
                    ack.order.details.push(split_line[4].to_string());
                    ack.order.details.push(split_line[2].to_string());
                }
                "DLV" => {
                    let tentative_ship_date = split_line[1];
                    let quantity_ordered = split_line[3].parse::<i32>().unwrap() / 1000;
                    let unit_price: f32 =
                        split_line[7].parse::<i32>().unwrap() as f32 / 100000 as f32;

                    ack.order.line_items.push(LineItem {
                        po1_segment: format!(
                            "PO1*{}*{}*{}*{}**VC*{}*BC*{}*",
                            ack.order.details[ack.order.details.len() - 3],
                            quantity_ordered.to_string(),
                            "CA".to_owned(),
                            unit_price.to_string(),
                            ack.order.details[ack.order.details.len() - 1],
                            ack.order.details[ack.order.details.len() - 2],
                        ),
                        ack_segment: format!(
                            "ACK*IA*{}*CA**{}",
                            quantity_ordered.to_string(),
                            tentative_ship_date.to_string()
                        ),
                    });
                    // line_order_array.clear();
                }
                _ => {
                    // do nothing
                }
            }
        }

        ack.control_tt_segment = format!("CTT*{}", ack.order.line_items.len());

        acks.push(ack);
    }

    let mut output_string: Vec<String> = Vec::new();

    for ack in acks {
        let mut line_items: String = "".to_owned();

        for line in &ack.order.line_items {
            line_items += format!("{}\n{}\n", line.po1_segment, line.ack_segment).as_str();
        }

        line_items = line_items[..line_items.len() - 1].to_string();

        let output = vec![
            ack.st_segment,
            ack.bak_segment,
            ack.n1_sf_segment,
            ack.n1_st_segment,
            // ack.n3_st_segment,
            // ack.n4_st_segment,
            line_items,
            ack.control_tt_segment,
            ack.se_segment,
        ]
        .join("\n");

        output_string.push(output);
    }

    for output in output_string {
        println!("{}", output);
    }
}
