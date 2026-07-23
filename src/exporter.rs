use crate::db::Database;
use crate::models::{InvestorFilter, LeadFilter, PersonFilter};
use rust_xlsxwriter::*;

pub fn generate_excel_export(db: &Database, filter: &LeadFilter) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x1E293B))
        .set_font_color(Color::RGB(0xFFFFFF));

    let high_format = Format::new()
        .set_background_color(Color::RGB(0xDCFCE7)) // Light green
        .set_font_color(Color::RGB(0x166534));

    let med_format = Format::new()
        .set_background_color(Color::RGB(0xFEF3C7)) // Light yellow
        .set_font_color(Color::RGB(0x92400E));

    // High Priority Worksheet
    let worksheet = workbook.add_worksheet().set_name("High Priority Leads")?;
    write_headers(worksheet, &header_format)?;

    let (leads, _) = db.get_leads(filter).unwrap_or((vec![], 0));

    let mut row_idx = 1;
    for company in &leads {
        if company.priority_tier == "HIGH" {
            write_company_row(worksheet, row_idx, company, Some(&high_format))?;
            row_idx += 1;
        }
    }

    // All Leads Worksheet
    let worksheet_all = workbook.add_worksheet().set_name("All Exported Leads")?;
    write_headers(worksheet_all, &header_format)?;

    row_idx = 1;
    for company in &leads {
        let fmt = match company.priority_tier.as_str() {
            "HIGH" => Some(&high_format),
            "MEDIUM" => Some(&med_format),
            _ => None,
        };
        write_company_row(worksheet_all, row_idx, company, fmt)?;
        row_idx += 1;
    }

    workbook.save_to_buffer()
}

pub fn generate_people_excel_export(db: &Database, filter: &PersonFilter) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x1E293B))
        .set_font_color(Color::RGB(0xFFFFFF));

    let cto_format = Format::new()
        .set_background_color(Color::RGB(0xDCFCE7))
        .set_font_color(Color::RGB(0x166534));

    let worksheet = workbook.add_worksheet().set_name("Decision Makers")?;
    let headers = vec![
        "ID", "Name", "Executive Title", "Normalized Role", "AI Score",
        "Company Name", "Company Domain", "Public Email", "LinkedIn Profile"
    ];

    for (col, text) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *text, &header_format)?;
    }

    let (people, _) = db.get_people(filter).unwrap_or((vec![], 0));

    let mut row_idx = 1;
    for p in &people {
        let fmt = if p.decision_maker_score >= 90 { Some(&cto_format) } else { None };

        worksheet.write_number(row_idx, 0, p.id as f64)?;
        worksheet.write_string(row_idx, 1, &p.name)?;
        worksheet.write_string(row_idx, 2, &p.title)?;
        worksheet.write_string(row_idx, 3, &p.normalized_role)?;

        if let Some(f) = fmt {
            worksheet.write_number_with_format(row_idx, 4, p.decision_maker_score as f64, f)?;
        } else {
            worksheet.write_number(row_idx, 4, p.decision_maker_score as f64)?;
        }

        worksheet.write_string(row_idx, 5, &p.company_name)?;
        worksheet.write_string(row_idx, 6, &p.company_domain)?;
        worksheet.write_string(row_idx, 7, p.public_email.as_deref().unwrap_or("-"))?;
        worksheet.write_string(row_idx, 8, p.linkedin_url.as_deref().unwrap_or("-"))?;

        row_idx += 1;
    }

    workbook.save_to_buffer()
}

pub fn generate_investor_excel_export(db: &Database, filter: &InvestorFilter) -> Result<Vec<u8>, XlsxError> {
    let mut workbook = Workbook::new();

    let header_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x1E293B))
        .set_font_color(Color::RGB(0xFFFFFF));

    let tier1_format = Format::new()
        .set_background_color(Color::RGB(0xDCFCE7))
        .set_font_color(Color::RGB(0x166534));

    let worksheet = workbook.add_worksheet().set_name("B2B SaaS & AI Investors")?;
    
    let headers = vec![
        "ID", "Investor Name", "Type", "Website", "Country", "City",
        "Investment Focus", "Stages", "Check Size", "Public Email",
        "LinkedIn", "Portfolio Highlights", "Recent Investments", "Score", "Tier"
    ];

    for (col, text) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *text, &header_format)?;
    }

    let (investors, _) = db.get_investors(filter).unwrap_or((vec![], 0));

    let mut row_idx = 1;
    for inv in &investors {
        let fmt = if inv.priority_tier == "TIER 1" { Some(&tier1_format) } else { None };

        let focus_str = inv.focus.join(", ");
        let stages_str = inv.stages.join(", ");
        let port_str = inv.portfolio_highlights.join(", ");
        let check_str = inv.check_size.as_deref().unwrap_or("-");
        let email_str = inv.public_email.as_deref().unwrap_or("-");
        let linkedin_str = inv.linkedin_url.as_deref().unwrap_or("-");
        let city_str = inv.city.as_deref().unwrap_or("-");

        worksheet.write_number(row_idx, 0, inv.id as f64)?;
        worksheet.write_string(row_idx, 1, &inv.name)?;
        worksheet.write_string(row_idx, 2, &inv.investor_type)?;
        worksheet.write_string(row_idx, 3, &inv.website)?;
        worksheet.write_string(row_idx, 4, &inv.country)?;
        worksheet.write_string(row_idx, 5, city_str)?;
        worksheet.write_string(row_idx, 6, &focus_str)?;
        worksheet.write_string(row_idx, 7, &stages_str)?;
        worksheet.write_string(row_idx, 8, check_str)?;
        worksheet.write_string(row_idx, 9, email_str)?;
        worksheet.write_string(row_idx, 10, linkedin_str)?;
        worksheet.write_string(row_idx, 11, &port_str)?;
        worksheet.write_number(row_idx, 12, inv.recent_investments as f64)?;

        if let Some(f) = fmt {
            worksheet.write_number_with_format(row_idx, 13, inv.score as f64, f)?;
            worksheet.write_string_with_format(row_idx, 14, &inv.priority_tier, f)?;
        } else {
            worksheet.write_number(row_idx, 13, inv.score as f64)?;
            worksheet.write_string(row_idx, 14, &inv.priority_tier)?;
        }

        row_idx += 1;
    }

    workbook.save_to_buffer()
}

fn write_headers(worksheet: &mut Worksheet, header_format: &Format) -> Result<(), XlsxError> {
    let headers = vec![
        "ID", "Company Name", "Domain", "Website", "Country", "City",
        "Industry", "Primary Email", "Contact Page",
        "LinkedIn", "Hiring Status", "Eng Jobs", "Remote Jobs",
        "Intent Signals", "Lead Score", "Priority Tier", "Tech Stack", "Last Crawled"
    ];

    for (col, text) in headers.iter().enumerate() {
        worksheet.write_string_with_format(0, col as u16, *text, header_format)?;
    }
    Ok(())
}

fn write_company_row(worksheet: &mut Worksheet, row: u32, c: &crate::models::Company, fmt: Option<&Format>) -> Result<(), XlsxError> {
    let tech_stack = c.tech_stack.join(", ");
    let hiring_str = if c.hiring { "YES" } else { "NO" };
    let email_str = c.email.as_deref().unwrap_or("-");
    let contact_str = c.contact_url.as_deref().unwrap_or("-");
    let linkedin_str = c.linkedin_url.as_deref().unwrap_or("-");
    let last_crawled_str = c.last_crawled.as_deref().unwrap_or("-");
    let city_str = c.city.as_deref().unwrap_or("-");
    let industry_str = c.industry.as_deref().unwrap_or("-");

    worksheet.write_number(row, 0, c.id as f64)?;
    worksheet.write_string(row, 1, &c.name)?;
    worksheet.write_string(row, 2, &c.domain)?;
    worksheet.write_string(row, 3, &c.website)?;
    worksheet.write_string(row, 4, &c.country)?;
    worksheet.write_string(row, 5, city_str)?;
    worksheet.write_string(row, 6, industry_str)?;
    worksheet.write_string(row, 7, email_str)?;
    worksheet.write_string(row, 8, contact_str)?;
    worksheet.write_string(row, 9, linkedin_str)?;
    worksheet.write_string(row, 10, hiring_str)?;
    worksheet.write_number(row, 11, c.engineering_jobs as f64)?;
    worksheet.write_number(row, 12, c.remote_jobs as f64)?;
    worksheet.write_number(row, 13, c.outsourcing_keywords as f64)?;
    
    if let Some(f) = fmt {
        worksheet.write_number_with_format(row, 14, c.lead_score as f64, f)?;
        worksheet.write_string_with_format(row, 15, &c.priority_tier, f)?;
    } else {
        worksheet.write_number(row, 14, c.lead_score as f64)?;
        worksheet.write_string(row, 15, &c.priority_tier)?;
    }

    worksheet.write_string(row, 16, &tech_stack)?;
    worksheet.write_string(row, 17, last_crawled_str)?;

    Ok(())
}
