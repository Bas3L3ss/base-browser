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
    Pre,               // <pre>
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
            "pre" => HtmlTag::Pre,

            // Fallback for unknown or custom tags
            other => HtmlTag::CustomTag(other.to_string()),
        }
    }

    /// Returns the HTML tag name string representation (e.g., "div", "h2", etc.).
    pub fn to_tag_string(&self) -> String {
        match self {
            HtmlTag::Doctype => "!doctype".to_string(),
            HtmlTag::Html => "html".to_string(),
            HtmlTag::Head => "head".to_string(),
            HtmlTag::Body => "body".to_string(),
            HtmlTag::Title => "title".to_string(),
            HtmlTag::Meta => "meta".to_string(),
            HtmlTag::Link => "link".to_string(),
            HtmlTag::Style => "style".to_string(),
            HtmlTag::Script => "script".to_string(), // Compatibility for variant called Script
            HtmlTag::Base => "base".to_string(),

            HtmlTag::HeaderTag => "header".to_string(),
            HtmlTag::Footer => "footer".to_string(),
            HtmlTag::Main => "main".to_string(),
            HtmlTag::Section => "section".to_string(),
            HtmlTag::Article => "article".to_string(),
            HtmlTag::Nav => "nav".to_string(),
            HtmlTag::Aside => "aside".to_string(),
            HtmlTag::Div => "div".to_string(),

            HtmlTag::Header(n) if *n >= 1 && *n <= 6 => format!("h{}", n),
            HtmlTag::Header(_) => "h".to_string(), // fallback

            HtmlTag::Paragraph => "p".to_string(),
            HtmlTag::Text => "#text".to_string(),
            HtmlTag::Whitespace => "#whitespace".to_string(),

            HtmlTag::OrderedList => "ol".to_string(),
            HtmlTag::UnorderedList => "ul".to_string(),
            HtmlTag::ListItem => "li".to_string(),
            HtmlTag::DescriptionList => "dl".to_string(),
            HtmlTag::DescriptionTerm => "dt".to_string(),
            HtmlTag::DescriptionDetails => "dd".to_string(),

            HtmlTag::Anchor => "a".to_string(),
            HtmlTag::Emphasis => "em".to_string(),
            HtmlTag::Strong => "strong".to_string(),
            HtmlTag::Italic => "i".to_string(),
            HtmlTag::Bold => "b".to_string(),
            HtmlTag::Small => "small".to_string(),
            HtmlTag::Mark => "mark".to_string(),
            HtmlTag::Delete => "del".to_string(),
            HtmlTag::Insert => "ins".to_string(),
            HtmlTag::Subscript => "sub".to_string(),
            HtmlTag::Superscript => "sup".to_string(),
            HtmlTag::Code => "code".to_string(),
            HtmlTag::Kbd => "kbd".to_string(),
            HtmlTag::Samp => "samp".to_string(),
            HtmlTag::Var => "var".to_string(),
            HtmlTag::Cite => "cite".to_string(),
            HtmlTag::Q => "q".to_string(),
            HtmlTag::Abbr => "abbr".to_string(),
            HtmlTag::Time => "time".to_string(),
            HtmlTag::Data => "data".to_string(),
            HtmlTag::Span => "span".to_string(),
            HtmlTag::Br => "br".to_string(),
            HtmlTag::Wbr => "wbr".to_string(),
            HtmlTag::Ruby => "ruby".to_string(),
            HtmlTag::Rt => "rt".to_string(),
            HtmlTag::Rp => "rp".to_string(),
            HtmlTag::Bdi => "bdi".to_string(),
            HtmlTag::Bdo => "bdo".to_string(),
            HtmlTag::S => "s".to_string(),
            HtmlTag::U => "u".to_string(),

            HtmlTag::Image => "img".to_string(),
            HtmlTag::Figure => "figure".to_string(),
            HtmlTag::Figcaption => "figcaption".to_string(),
            HtmlTag::Audio => "audio".to_string(),
            HtmlTag::Video => "video".to_string(),
            HtmlTag::Source => "source".to_string(),
            HtmlTag::Track => "track".to_string(),
            HtmlTag::Canvas => "canvas".to_string(),
            HtmlTag::Map => "map".to_string(),
            HtmlTag::Area => "area".to_string(),
            HtmlTag::Picture => "picture".to_string(),
            HtmlTag::Iframe => "iframe".to_string(),
            HtmlTag::Embed => "embed".to_string(),
            HtmlTag::Object => "object".to_string(),
            HtmlTag::Param => "param".to_string(),

            HtmlTag::Table => "table".to_string(),
            HtmlTag::TableRow => "tr".to_string(),
            HtmlTag::TableData => "td".to_string(),
            HtmlTag::TableHeader => "th".to_string(),
            HtmlTag::TableHead => "thead".to_string(),
            HtmlTag::TableBody => "tbody".to_string(),
            HtmlTag::TableFoot => "tfoot".to_string(),
            HtmlTag::Colgroup => "colgroup".to_string(),
            HtmlTag::Col => "col".to_string(),
            HtmlTag::Caption => "caption".to_string(),

            HtmlTag::Form => "form".to_string(),
            HtmlTag::Input => "input".to_string(),
            HtmlTag::Button => "button".to_string(),
            HtmlTag::Textarea => "textarea".to_string(),
            HtmlTag::Select => "select".to_string(),
            HtmlTag::Option => "option".to_string(),
            HtmlTag::Optgroup => "optgroup".to_string(),
            HtmlTag::Label => "label".to_string(),
            HtmlTag::Fieldset => "fieldset".to_string(),
            HtmlTag::Legend => "legend".to_string(),
            HtmlTag::Datalist => "datalist".to_string(),
            HtmlTag::Output => "output".to_string(),
            HtmlTag::Progress => "progress".to_string(),
            HtmlTag::Meter => "meter".to_string(),

            HtmlTag::ScriptTag => "script".to_string(),
            HtmlTag::Noscript => "noscript".to_string(),
            HtmlTag::Template => "template".to_string(),
            HtmlTag::Slot => "slot".to_string(),
            HtmlTag::CanvasTag => "canvas".to_string(),

            HtmlTag::Details => "details".to_string(),
            HtmlTag::Summary => "summary".to_string(),
            HtmlTag::Dialog => "dialog".to_string(),

            HtmlTag::Pre => "pre".to_string(),

            HtmlTag::CustomTag(s) => s.clone(),
        }
    }
}
