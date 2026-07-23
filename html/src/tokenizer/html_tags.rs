#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlTag {
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
    Div, // <div>

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
    CustomTag(String), // Any unknown tag for extensibility
}

impl HtmlTag {
    /// Create an `HtmlTag` variant from a tag name string (e.g., "p", "h2", "html", "random-tag")
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            // Document Structure
            "!doctype" => HtmlTag::Doctype,
            "html" => HtmlTag::Html,
            "head" => HtmlTag::Head,
            "body" => HtmlTag::Body,

            // Metadata
            "title" => HtmlTag::Title,
            "meta" => HtmlTag::Meta,
            "link" => HtmlTag::Link,
            "style" => HtmlTag::Style,
            "script" => HtmlTag::ScriptTag,
            "base" => HtmlTag::Base,

            // Sections
            "header" => HtmlTag::HeaderTag,
            "footer" => HtmlTag::Footer,
            "main" => HtmlTag::Main,
            "section" => HtmlTag::Section,
            "article" => HtmlTag::Article,
            "nav" => HtmlTag::Nav,
            "aside" => HtmlTag::Aside,
            "div" => HtmlTag::Div,

            // Headings h1-h6
            tag if tag.starts_with('h')
                && tag.len() == 2
                && tag.chars().nth(1).unwrap().is_ascii_digit() =>
            {
                match tag[1..2].parse::<u8>() {
                    Ok(n @ 1..=6) => HtmlTag::Header(n),
                    _ => HtmlTag::CustomTag(s.to_string()),
                }
            }

            // Text content
            "p" => HtmlTag::Paragraph,
            "#text" => HtmlTag::Text,
            "#whitespace" => HtmlTag::Whitespace,

            // Lists
            "ol" => HtmlTag::OrderedList,
            "ul" => HtmlTag::UnorderedList,
            "li" => HtmlTag::ListItem,
            "dl" => HtmlTag::DescriptionList,
            "dt" => HtmlTag::DescriptionTerm,
            "dd" => HtmlTag::DescriptionDetails,

            // Inline text semantics
            "a" => HtmlTag::Anchor,
            "em" => HtmlTag::Emphasis,
            "strong" => HtmlTag::Strong,
            "i" => HtmlTag::Italic,
            "b" => HtmlTag::Bold,
            "small" => HtmlTag::Small,
            "mark" => HtmlTag::Mark,
            "del" => HtmlTag::Delete,
            "ins" => HtmlTag::Insert,
            "sub" => HtmlTag::Subscript,
            "sup" => HtmlTag::Superscript,
            "code" => HtmlTag::Code,
            "kbd" => HtmlTag::Kbd,
            "samp" => HtmlTag::Samp,
            "var" => HtmlTag::Var,
            "cite" => HtmlTag::Cite,
            "q" => HtmlTag::Q,
            "abbr" => HtmlTag::Abbr,
            "time" => HtmlTag::Time,
            "data" => HtmlTag::Data,
            "span" => HtmlTag::Span,
            "br" => HtmlTag::Br,
            "wbr" => HtmlTag::Wbr,
            // More phrasing tags
            "ruby" => HtmlTag::Ruby,
            "rt" => HtmlTag::Rt,
            "rp" => HtmlTag::Rp,
            "bdi" => HtmlTag::Bdi,
            "bdo" => HtmlTag::Bdo,
            "s" => HtmlTag::S,
            "u" => HtmlTag::U,

            // Image and multimedia
            "img" => HtmlTag::Image,
            "figure" => HtmlTag::Figure,
            "figcaption" => HtmlTag::Figcaption,
            "audio" => HtmlTag::Audio,
            "video" => HtmlTag::Video,
            "source" => HtmlTag::Source,
            "track" => HtmlTag::Track,
            "canvas" => HtmlTag::CanvasTag,
            "map" => HtmlTag::Map,
            "area" => HtmlTag::Area,
            "picture" => HtmlTag::Picture,
            "iframe" => HtmlTag::Iframe,
            "embed" => HtmlTag::Embed,
            "object" => HtmlTag::Object,
            "param" => HtmlTag::Param,

            // Tables
            "table" => HtmlTag::Table,
            "tr" => HtmlTag::TableRow,
            "td" => HtmlTag::TableData,
            "th" => HtmlTag::TableHeader,
            "thead" => HtmlTag::TableHead,
            "tbody" => HtmlTag::TableBody,
            "tfoot" => HtmlTag::TableFoot,
            "colgroup" => HtmlTag::Colgroup,
            "col" => HtmlTag::Col,
            "caption" => HtmlTag::Caption,

            // Forms
            "form" => HtmlTag::Form,
            "input" => HtmlTag::Input,
            "button" => HtmlTag::Button,
            "textarea" => HtmlTag::Textarea,
            "select" => HtmlTag::Select,
            "option" => HtmlTag::Option,
            "optgroup" => HtmlTag::Optgroup,
            "label" => HtmlTag::Label,
            "fieldset" => HtmlTag::Fieldset,
            "legend" => HtmlTag::Legend,
            "datalist" => HtmlTag::Datalist,
            "output" => HtmlTag::Output,
            "progress" => HtmlTag::Progress,
            "meter" => HtmlTag::Meter,

            // Scripting
            "noscript" => HtmlTag::Noscript,
            "template" => HtmlTag::Template,
            "slot" => HtmlTag::Slot,

            // Other common tags
            "details" => HtmlTag::Details,
            "summary" => HtmlTag::Summary,
            "dialog" => HtmlTag::Dialog,

            // Fallback for unknown or custom tags
            other => HtmlTag::CustomTag(other.to_string()),
        }
    }
}
