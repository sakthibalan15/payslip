// backend/src/pdf.rs
use printpdf::*;
use std::io::BufWriter;

pub struct PayslipData {
    pub employee_name:   String,
    pub employee_id:     String,
    pub employee_email:  String,
    pub department:      String,
    pub designation:     String,
    pub year:            i32,
    pub month:           i32,
    pub basic:           f64,
    pub hra:             f64,
    pub conveyance:      f64,
    pub other_allowance: f64,
    pub pf_deduction:    f64,
    pub tax_deduction:   f64,
    pub other_deduction: f64,
    pub net_pay:         f64,
}

impl PayslipData {
    fn gross(&self) -> f64 { self.basic + self.hra + self.conveyance + self.other_allowance }
    fn deductions(&self) -> f64 { self.pf_deduction + self.tax_deduction + self.other_deduction }
    fn month_name(&self) -> &'static str {
        ["January","February","March","April","May","June",
         "July","August","September","October","November","December"]
            [(self.month as usize).saturating_sub(1).min(11)]
    }
}

pub fn generate(d: &PayslipData) -> Vec<u8> {
    let (doc, p1, l1) = PdfDocument::new("Payslip", Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(p1).get_layer(l1);
    let bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
    let reg  = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();

    // Header
    layer.use_text("COMPANY NAME",            20.0, Mm(70.0), Mm(275.0), &bold);
    layer.use_text("PAYSLIP",                 14.0, Mm(88.0), Mm(265.0), &bold);
    layer.use_text(&format!("{} {}",d.month_name(),d.year), 11.0, Mm(80.0), Mm(257.0), &reg);

    // Divider
    layer.add_line(Line {
        points: vec![(Point::new(Mm(15.0),Mm(252.0)),false),(Point::new(Mm(195.0),Mm(252.0)),false)],
        is_closed: false,
    });

    // Employee info
    let mut y = 244.0f64;
    for (lbl,val) in [
        ("Employee Name:", d.employee_name.as_str()),
        ("Employee ID:",   d.employee_id.as_str()),
        ("Email:",         d.employee_email.as_str()),
        ("Department:",    d.department.as_str()),
        ("Designation:",   d.designation.as_str()),
    ] {
        layer.use_text(lbl, 10.0, Mm(15.0), Mm(y), &bold);
        layer.use_text(val, 10.0, Mm(75.0), Mm(y), &reg);
        y -= 8.0;
    }

    // Earnings / Deductions
    y -= 6.0;
    for (t,x) in [("EARNINGS",15.0),("AMOUNT (INR)",80.0),("DEDUCTIONS",120.0),("AMOUNT (INR)",175.0)] {
        layer.use_text(t, 11.0, Mm(x), Mm(y), &bold);
    }
    y -= 6.0;

    let earn = [("Basic Salary",d.basic),("HRA",d.hra),("Conveyance",d.conveyance),("Other Allowance",d.other_allowance)];
    let dedu = [("PF Deduction",d.pf_deduction),("Tax (TDS)",d.tax_deduction),("Other",d.other_deduction)];

    for (i,(elbl,eamt)) in earn.iter().enumerate() {
        layer.use_text(elbl, 10.0, Mm(15.0),  Mm(y), &reg);
        layer.use_text(&format!("{eamt:.2}"), 10.0, Mm(80.0), Mm(y), &reg);
        if let Some((dlbl,damt)) = dedu.get(i) {
            layer.use_text(dlbl, 10.0, Mm(120.0), Mm(y), &reg);
            layer.use_text(&format!("{damt:.2}"), 10.0, Mm(175.0), Mm(y), &reg);
        }
        y -= 8.0;
    }

    y -= 4.0;
    layer.use_text("Gross Earnings:",    10.0, Mm(15.0),  Mm(y), &bold);
    layer.use_text(&format!("{:.2}",d.gross()), 10.0, Mm(80.0), Mm(y), &bold);
    layer.use_text("Total Deductions:", 10.0, Mm(120.0), Mm(y), &bold);
    layer.use_text(&format!("{:.2}",d.deductions()), 10.0, Mm(175.0), Mm(y), &bold);

    y -= 16.0;
    layer.use_text(&format!("NET PAY: INR {:.2}", d.net_pay), 14.0, Mm(65.0), Mm(y), &bold);
    layer.use_text("System-generated payslip. No signature required.", 8.0, Mm(40.0), Mm(20.0), &reg);

    let mut buf = BufWriter::new(Vec::new());
    doc.save(&mut buf).unwrap();
    buf.into_inner().unwrap()
}
