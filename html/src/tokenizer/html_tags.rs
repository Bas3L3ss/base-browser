#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlTags {
    // Document Structure
    Doctype,
    Html,
    Head,
    Body,

    // Metadata
    Title,
    Meta,
    Link,
    Style,
    Script,
    Base,

    // Sections
    HeaderTag, // <header>
    Footer,
    Main,
    Section,
    Article,
    Nav,
    Aside,

    // Headings
    Header(u8), // Header(1) means H1, Header(2) means H2, etc.

    // Text content
    Paragraph, // <p>
    Text,
    Whitespace,

    // Lists
    OrderedList,        // <ol>
    UnorderedList,      // <ul>
    ListItem,           // <li>
    DescriptionList,    // <dl>
    DescriptionTerm,    // <dt>
    DescriptionDetails, // <dd>

    // Inline text semantics
    Anchor,   // <a>
    Emphasis, // <em>
    Strong,   // <strong>
    Italic,   // <i>
    Bold,     // <b>
    Small,
    Mark,
    Delete,      // <del>
    Insert,      // <ins>
    Subscript,   // <sub>
    Superscript, // <sup>
    Code,
    Kbd,
    Samp,
    Var,
    Cite,
    Q,
    Abbr,
    Time,
    Data,
    Span, // <span>
    Br,
    Wbr,
    // Extend with more inline/phrasing tags as needed
    // Ruby annotation tags (some modern HTML)
    Ruby,
    Rt,
    Rp,
    Bdi,
    Bdo,
    S, // <s>
    U, // <u>

    // Image and multimedia
    Image, // <img>
    Figure,
    Figcaption,
    Audio,
    Video,
    Source,
    Track,
    Canvas,
    Map,
    Area,
    Picture,
    Iframe,
    Embed,
    Object,
    Param,

    // Tables
    Table,
    TableRow,    // <tr>
    TableData,   // <td>
    TableHeader, // <th>
    TableHead,   // <thead>
    TableBody,   // <tbody>
    TableFoot,   // <tfoot>
    Colgroup,
    Col,
    Caption, // <caption>

    // Forms
    Form,
    Input,
    Button,
    Textarea,
    Select,
    Option,
    Optgroup,
    Label,
    Fieldset,
    Legend,
    Datalist,
    Output,
    Progress,
    Meter,

    // Scripting
    ScriptTag, // <script>
    Noscript,
    Template,
    Slot,
    CanvasTag, // <canvas>

    // Other common tags
    Details,
    Summary,
    Dialog,

    // Others and special
    UnknownTag(String), // Any unknown tag for extensibility
}

impl HtmlTags {
    /// Create an `HtmlTags` variant from a tag name string (e.g., "p", "h2", "html", "random-tag")
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            // Document Structure
            "!doctype" => HtmlTags::Doctype,
            "html" => HtmlTags::Html,
            "head" => HtmlTags::Head,
            "body" => HtmlTags::Body,

            // Metadata
            "title" => HtmlTags::Title,
            "meta" => HtmlTags::Meta,
            "link" => HtmlTags::Link,
            "style" => HtmlTags::Style,
            "script" => HtmlTags::ScriptTag,
            "base" => HtmlTags::Base,

            // Sections
            "header" => HtmlTags::HeaderTag,
            "footer" => HtmlTags::Footer,
            "main" => HtmlTags::Main,
            "section" => HtmlTags::Section,
            "article" => HtmlTags::Article,
            "nav" => HtmlTags::Nav,
            "aside" => HtmlTags::Aside,

            // Headings h1-h6
            tag if tag.starts_with('h')
                && tag.len() == 2
                && tag.chars().nth(1).unwrap().is_ascii_digit() =>
            {
                match tag[1..2].parse::<u8>() {
                    Ok(n @ 1..=6) => HtmlTags::Header(n),
                    _ => HtmlTags::UnknownTag(s.to_string()),
                }
            }

            // Text content
            "p" => HtmlTags::Paragraph,
            "#text" => HtmlTags::Text,
            "#whitespace" => HtmlTags::Whitespace,

            // Lists
            "ol" => HtmlTags::OrderedList,
            "ul" => HtmlTags::UnorderedList,
            "li" => HtmlTags::ListItem,
            "dl" => HtmlTags::DescriptionList,
            "dt" => HtmlTags::DescriptionTerm,
            "dd" => HtmlTags::DescriptionDetails,

            // Inline text semantics
            "a" => HtmlTags::Anchor,
            "em" => HtmlTags::Emphasis,
            "strong" => HtmlTags::Strong,
            "i" => HtmlTags::Italic,
            "b" => HtmlTags::Bold,
            "small" => HtmlTags::Small,
            "mark" => HtmlTags::Mark,
            "del" => HtmlTags::Delete,
            "ins" => HtmlTags::Insert,
            "sub" => HtmlTags::Subscript,
            "sup" => HtmlTags::Superscript,
            "code" => HtmlTags::Code,
            "kbd" => HtmlTags::Kbd,
            "samp" => HtmlTags::Samp,
            "var" => HtmlTags::Var,
            "cite" => HtmlTags::Cite,
            "q" => HtmlTags::Q,
            "abbr" => HtmlTags::Abbr,
            "time" => HtmlTags::Time,
            "data" => HtmlTags::Data,
            "span" => HtmlTags::Span,
            "br" => HtmlTags::Br,
            "wbr" => HtmlTags::Wbr,
            // More phrasing tags
            "ruby" => HtmlTags::Ruby,
            "rt" => HtmlTags::Rt,
            "rp" => HtmlTags::Rp,
            "bdi" => HtmlTags::Bdi,
            "bdo" => HtmlTags::Bdo,
            "s" => HtmlTags::S,
            "u" => HtmlTags::U,

            // Image and multimedia
            "img" => HtmlTags::Image,
            "figure" => HtmlTags::Figure,
            "figcaption" => HtmlTags::Figcaption,
            "audio" => HtmlTags::Audio,
            "video" => HtmlTags::Video,
            "source" => HtmlTags::Source,
            "track" => HtmlTags::Track,
            "canvas" => HtmlTags::CanvasTag,
            "map" => HtmlTags::Map,
            "area" => HtmlTags::Area,
            "picture" => HtmlTags::Picture,
            "iframe" => HtmlTags::Iframe,
            "embed" => HtmlTags::Embed,
            "object" => HtmlTags::Object,
            "param" => HtmlTags::Param,

            // Tables
            "table" => HtmlTags::Table,
            "tr" => HtmlTags::TableRow,
            "td" => HtmlTags::TableData,
            "th" => HtmlTags::TableHeader,
            "thead" => HtmlTags::TableHead,
            "tbody" => HtmlTags::TableBody,
            "tfoot" => HtmlTags::TableFoot,
            "colgroup" => HtmlTags::Colgroup,
            "col" => HtmlTags::Col,
            "caption" => HtmlTags::Caption,

            // Forms
            "form" => HtmlTags::Form,
            "input" => HtmlTags::Input,
            "button" => HtmlTags::Button,
            "textarea" => HtmlTags::Textarea,
            "select" => HtmlTags::Select,
            "option" => HtmlTags::Option,
            "optgroup" => HtmlTags::Optgroup,
            "label" => HtmlTags::Label,
            "fieldset" => HtmlTags::Fieldset,
            "legend" => HtmlTags::Legend,
            "datalist" => HtmlTags::Datalist,
            "output" => HtmlTags::Output,
            "progress" => HtmlTags::Progress,
            "meter" => HtmlTags::Meter,

            // Scripting
            "noscript" => HtmlTags::Noscript,
            "template" => HtmlTags::Template,
            "slot" => HtmlTags::Slot,

            // Other common tags
            "details" => HtmlTags::Details,
            "summary" => HtmlTags::Summary,
            "dialog" => HtmlTags::Dialog,

            // Fallback for unknown or custom tags
            other => HtmlTags::UnknownTag(other.to_string()),
        }
    }
}
