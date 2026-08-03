#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Property {
    // Layout
    Display,
    Position,
    Top,
    Right,
    Bottom,
    Left,
    Float,
    Clear,
    Inset,
    InsetBlock,
    InsetBlockStart,
    InsetBlockEnd,
    InsetInline,
    InsetInlineStart,
    InsetInlineEnd,

    // Box Model
    Width,
    Height,
    MinWidth,
    MaxWidth,
    MinHeight,
    MaxHeight,
    Margin,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    MarginBlock,
    MarginBlockStart,
    MarginBlockEnd,
    MarginInline,
    MarginInlineStart,
    MarginInlineEnd,
    Padding,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    PaddingBlock,
    PaddingBlockStart,
    PaddingBlockEnd,
    PaddingInline,
    PaddingInlineStart,
    PaddingInlineEnd,
    Border,
    BorderTop,
    BorderRight,
    BorderBottom,
    BorderLeft,
    BorderBlock,
    BorderBlockStart,
    BorderBlockEnd,
    BorderInline,
    BorderInlineStart,
    BorderInlineEnd,
    BorderWidth,
    BorderStyle,
    BorderColor,
    BorderRadius,
    BorderTopLeftRadius,
    BorderTopRightRadius,
    BorderBottomLeftRadius,
    BorderBottomRightRadius,
    BoxSizing,
    Outline,
    OutlineWidth,
    OutlineColor,
    OutlineStyle,
    BoxDecorationBreak,

    // Shorthands for box model
    BorderBlockRadius,
    BorderEndEndRadius,
    BorderEndStartRadius,
    BorderStartEndRadius,
    BorderStartStartRadius,

    // Background
    Background,
    BackgroundColor,
    BackgroundImage,
    BackgroundPosition,
    BackgroundRepeat,
    BackgroundSize,
    BackgroundAttachment,
    BackgroundClip,
    BackgroundOrigin,
    BackgroundBlendMode,

    // Media
    Src,

    // Typography
    Font,
    FontSize,
    FontWeight,
    FontFamily,
    FontStyle,
    FontVariant,
    FontStretch,
    LineHeight,
    LetterSpacing,
    WordSpacing,
    TextAlign,
    TextDecoration,
    TextDecorationColor,
    TextDecorationStyle,
    TextDecorationLine,
    TextDecorationThickness,
    TextTransform,
    TextIndent,
    TextOverflow,
    TextShadow,
    Color,
    FontFeatureSettings,
    FontKerning,
    FontOpticalSizing,
    FontPalette,
    FontSynthesis,
    FontVariationSettings,
    FontVariantCaps,
    FontVariantLigatures,
    FontVariantNumeric,
    Direction,
    UnicodeBidi,

    // Effects
    Opacity,
    BoxShadow,
    Filter,
    BackdropFilter,
    MixBlendMode,
    Isolation,

    // Overflow & Visibility
    Overflow,
    OverflowX,
    OverflowY,
    OverflowWrap,
    WordBreak,
    WhiteSpace,
    Visibility,
    Cursor,
    PointerEvents,
    UserSelect,
    CaretColor,
    Clip,
    ClipPath,
    ScrollBehavior,
    WillChange,
    Resize,
    OverflowAnchor,

    // Flexbox
    Flex,
    FlexDirection,
    FlexWrap,
    FlexFlow,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    JustifyContent,
    AlignItems,
    AlignContent,
    AlignSelf,
    Order,
    Gap,
    RowGap,
    ColumnGap,
    PlaceItems,
    PlaceSelf,

    // Grid
    DisplayGrid,
    Grid,
    GridTemplate,
    GridTemplateColumns,
    GridTemplateRows,
    GridTemplateAreas,
    GridColumn,
    GridRow,
    GridArea,
    GridColumnStart,
    GridColumnEnd,
    GridRowStart,
    GridRowEnd,
    GridAutoColumns,
    GridAutoRows,
    GridAutoFlow,
    GridGap,
    GridColumnGap,
    GridRowGap,
    JustifyItems,
    JustifySelf,

    // Positioning and Layering
    ObjectFit,
    ObjectPosition,
    FloatDirection,
    TopInset,
    BottomInset,
    RightInset,
    LeftInset,
    ZIndex,

    // Animation & Transition
    Animation,
    AnimationName,
    AnimationDuration,
    AnimationTimingFunction,
    AnimationDelay,
    AnimationIterationCount,
    AnimationDirection,
    AnimationFillMode,
    AnimationPlayState,
    Transition,
    TransitionProperty,
    TransitionDuration,
    TransitionTimingFunction,
    TransitionDelay,

    // List
    ListStyle,
    ListStyleType,
    ListStylePosition,
    ListStyleImage,

    // Table
    BorderCollapse,
    BorderSpacing,
    TableLayout,
    EmptyCells,
    CaptionSide,
    VerticalAlign,

    // Content & Generation
    Content,
    Quotes,
    CounterReset,
    CounterIncrement,
    ContentVisibility,
    BreakAfter,
    BreakBefore,
    BreakInside,

    // Misc
    TabSize,
    VisibilityCollapse,
    BackfaceVisibility,
    Perspective,
    PerspectiveOrigin,
    Transform,
    TransformOrigin,
    Appearance,
    AccentColor,
    ScrollMargin,
    ScrollMarginTop,
    ScrollMarginRight,
    ScrollMarginBottom,
    ScrollMarginLeft,
    ScrollMarginBlock,
    ScrollMarginBlockStart,
    ScrollMarginBlockEnd,
    ScrollMarginInline,
    ScrollMarginInlineStart,
    ScrollMarginInlineEnd,
    ScrollPadding,
    ScrollPaddingTop,
    ScrollPaddingRight,
    ScrollPaddingBottom,
    ScrollPaddingLeft,
    ScrollPaddingBlock,
    ScrollPaddingBlockStart,
    ScrollPaddingBlockEnd,
    ScrollPaddingInline,
    ScrollPaddingInlineStart,
    ScrollPaddingInlineEnd,

    // Custom or not yet listed
    Unknown(String),
}

impl Property {
    pub fn from_str(s: &str) -> Self {
        match s {
            // Layout
            "display" => Property::Display,
            "position" => Property::Position,
            "top" => Property::Top,
            "bottom" => Property::Bottom,
            "right" => Property::Right,
            "left" => Property::Left,
            "z-index" => Property::ZIndex,
            "float" => Property::Float,
            "clear" => Property::Clear,
            "inset" => Property::Inset,
            "inset-block" => Property::InsetBlock,
            "inset-block-start" => Property::InsetBlockStart,
            "inset-block-end" => Property::InsetBlockEnd,
            "inset-inline" => Property::InsetInline,
            "inset-inline-start" => Property::InsetInlineStart,
            "inset-inline-end" => Property::InsetInlineEnd,

            // Box Model
            "width" => Property::Width,
            "height" => Property::Height,
            "min-width" => Property::MinWidth,
            "max-width" => Property::MaxWidth,
            "min-height" => Property::MinHeight,
            "max-height" => Property::MaxHeight,
            "margin" => Property::Margin,
            "margin-top" => Property::MarginTop,
            "margin-right" => Property::MarginRight,
            "margin-bottom" => Property::MarginBottom,
            "margin-left" => Property::MarginLeft,
            "margin-block" => Property::MarginBlock,
            "margin-block-start" => Property::MarginBlockStart,
            "margin-block-end" => Property::MarginBlockEnd,
            "margin-inline" => Property::MarginInline,
            "margin-inline-start" => Property::MarginInlineStart,
            "margin-inline-end" => Property::MarginInlineEnd,
            "padding" => Property::Padding,
            "padding-top" => Property::PaddingTop,
            "padding-right" => Property::PaddingRight,
            "padding-bottom" => Property::PaddingBottom,
            "padding-left" => Property::PaddingLeft,
            "padding-block" => Property::PaddingBlock,
            "padding-block-start" => Property::PaddingBlockStart,
            "padding-block-end" => Property::PaddingBlockEnd,
            "padding-inline" => Property::PaddingInline,
            "padding-inline-start" => Property::PaddingInlineStart,
            "padding-inline-end" => Property::PaddingInlineEnd,
            "border" => Property::Border,
            "border-top" => Property::BorderTop,
            "border-right" => Property::BorderRight,
            "border-bottom" => Property::BorderBottom,
            "border-left" => Property::BorderLeft,
            "border-block" => Property::BorderBlock,
            "border-block-start" => Property::BorderBlockStart,
            "border-block-end" => Property::BorderBlockEnd,
            "border-inline" => Property::BorderInline,
            "border-inline-start" => Property::BorderInlineStart,
            "border-inline-end" => Property::BorderInlineEnd,
            "border-width" => Property::BorderWidth,
            "border-style" => Property::BorderStyle,
            "border-color" => Property::BorderColor,
            "border-radius" => Property::BorderRadius,
            "border-top-left-radius" => Property::BorderTopLeftRadius,
            "border-top-right-radius" => Property::BorderTopRightRadius,
            "border-bottom-left-radius" => Property::BorderBottomLeftRadius,
            "border-bottom-right-radius" => Property::BorderBottomRightRadius,
            "box-sizing" => Property::BoxSizing,
            "outline" => Property::Outline,
            "outline-width" => Property::OutlineWidth,
            "outline-color" => Property::OutlineColor,
            "outline-style" => Property::OutlineStyle,
            "box-decoration-break" => Property::BoxDecorationBreak,
            "border-block-radius" => Property::BorderBlockRadius,
            "border-end-end-radius" => Property::BorderEndEndRadius,
            "border-end-start-radius" => Property::BorderEndStartRadius,
            "border-start-end-radius" => Property::BorderStartEndRadius,
            "border-start-start-radius" => Property::BorderStartStartRadius,

            // Background
            "background" => Property::Background,
            "background-color" => Property::BackgroundColor,
            "background-image" => Property::BackgroundImage,
            "background-position" => Property::BackgroundPosition,
            "background-repeat" => Property::BackgroundRepeat,
            "background-size" => Property::BackgroundSize,
            "background-attachment" => Property::BackgroundAttachment,
            "background-clip" => Property::BackgroundClip,
            "background-origin" => Property::BackgroundOrigin,
            "background-blend-mode" => Property::BackgroundBlendMode,

            // Media
            "src" => Property::Src,

            // Typography
            "font" => Property::Font,
            "font-size" => Property::FontSize,
            "font-weight" => Property::FontWeight,
            "font-family" => Property::FontFamily,
            "font-style" => Property::FontStyle,
            "font-variant" => Property::FontVariant,
            "font-stretch" => Property::FontStretch,
            "line-height" => Property::LineHeight,
            "letter-spacing" => Property::LetterSpacing,
            "word-spacing" => Property::WordSpacing,
            "text-align" => Property::TextAlign,
            "text-decoration" => Property::TextDecoration,
            "text-decoration-color" => Property::TextDecorationColor,
            "text-decoration-style" => Property::TextDecorationStyle,
            "text-decoration-line" => Property::TextDecorationLine,
            "text-decoration-thickness" => Property::TextDecorationThickness,
            "text-transform" => Property::TextTransform,
            "text-indent" => Property::TextIndent,
            "text-overflow" => Property::TextOverflow,
            "text-shadow" => Property::TextShadow,
            "color" => Property::Color,
            "font-feature-settings" => Property::FontFeatureSettings,
            "font-kerning" => Property::FontKerning,
            "font-optical-sizing" => Property::FontOpticalSizing,
            "font-palette" => Property::FontPalette,
            "font-synthesis" => Property::FontSynthesis,
            "font-variation-settings" => Property::FontVariationSettings,
            "font-variant-caps" => Property::FontVariantCaps,
            "font-variant-ligatures" => Property::FontVariantLigatures,
            "font-variant-numeric" => Property::FontVariantNumeric,
            "direction" => Property::Direction,
            "unicode-bidi" => Property::UnicodeBidi,

            // Effects
            "opacity" => Property::Opacity,
            "box-shadow" => Property::BoxShadow,
            "filter" => Property::Filter,
            "backdrop-filter" => Property::BackdropFilter,
            "mix-blend-mode" => Property::MixBlendMode,
            "isolation" => Property::Isolation,

            // Overflow & Visibility
            "overflow" => Property::Overflow,
            "overflow-x" => Property::OverflowX,
            "overflow-y" => Property::OverflowY,
            "overflow-wrap" => Property::OverflowWrap,
            "word-break" => Property::WordBreak,
            "white-space" => Property::WhiteSpace,
            "visibility" => Property::Visibility,
            "cursor" => Property::Cursor,
            "pointer-events" => Property::PointerEvents,
            "user-select" => Property::UserSelect,
            "caret-color" => Property::CaretColor,
            "clip" => Property::Clip,
            "clip-path" => Property::ClipPath,
            "scroll-behavior" => Property::ScrollBehavior,
            "will-change" => Property::WillChange,
            "resize" => Property::Resize,
            "overflow-anchor" => Property::OverflowAnchor,

            // Flexbox
            "flex" => Property::Flex,
            "flex-direction" => Property::FlexDirection,
            "flex-wrap" => Property::FlexWrap,
            "flex-flow" => Property::FlexFlow,
            "flex-grow" => Property::FlexGrow,
            "flex-shrink" => Property::FlexShrink,
            "flex-basis" => Property::FlexBasis,
            "justify-content" => Property::JustifyContent,
            "align-items" => Property::AlignItems,
            "align-content" => Property::AlignContent,
            "align-self" => Property::AlignSelf,
            "order" => Property::Order,
            "gap" => Property::Gap,
            "row-gap" => Property::RowGap,
            "column-gap" => Property::ColumnGap,
            "place-items" => Property::PlaceItems,
            "place-self" => Property::PlaceSelf,

            // Grid
            "display-grid" => Property::DisplayGrid,
            "grid" => Property::Grid,
            "grid-template" => Property::GridTemplate,
            "grid-template-columns" => Property::GridTemplateColumns,
            "grid-template-rows" => Property::GridTemplateRows,
            "grid-template-areas" => Property::GridTemplateAreas,
            "grid-column" => Property::GridColumn,
            "grid-row" => Property::GridRow,
            "grid-area" => Property::GridArea,
            "grid-column-start" => Property::GridColumnStart,
            "grid-column-end" => Property::GridColumnEnd,
            "grid-row-start" => Property::GridRowStart,
            "grid-row-end" => Property::GridRowEnd,
            "grid-auto-columns" => Property::GridAutoColumns,
            "grid-auto-rows" => Property::GridAutoRows,
            "grid-auto-flow" => Property::GridAutoFlow,
            "grid-gap" => Property::GridGap,
            "grid-column-gap" => Property::GridColumnGap,
            "grid-row-gap" => Property::GridRowGap,
            "justify-items" => Property::JustifyItems,
            "justify-self" => Property::JustifySelf,

            // Positioning and Layering
            "object-fit" => Property::ObjectFit,
            "object-position" => Property::ObjectPosition,
            "float-direction" => Property::FloatDirection,
            "inset-top" => Property::TopInset,
            "inset-bottom" => Property::BottomInset,
            "inset-right" => Property::RightInset,
            "inset-left" => Property::LeftInset,

            // Animation & Transition
            "animation" => Property::Animation,
            "animation-name" => Property::AnimationName,
            "animation-duration" => Property::AnimationDuration,
            "animation-timing-function" => Property::AnimationTimingFunction,
            "animation-delay" => Property::AnimationDelay,
            "animation-iteration-count" => Property::AnimationIterationCount,
            "animation-direction" => Property::AnimationDirection,
            "animation-fill-mode" => Property::AnimationFillMode,
            "animation-play-state" => Property::AnimationPlayState,
            "transition" => Property::Transition,
            "transition-property" => Property::TransitionProperty,
            "transition-duration" => Property::TransitionDuration,
            "transition-timing-function" => Property::TransitionTimingFunction,
            "transition-delay" => Property::TransitionDelay,

            // List
            "list-style" => Property::ListStyle,
            "list-style-type" => Property::ListStyleType,
            "list-style-position" => Property::ListStylePosition,
            "list-style-image" => Property::ListStyleImage,

            // Table
            "border-collapse" => Property::BorderCollapse,
            "border-spacing" => Property::BorderSpacing,
            "table-layout" => Property::TableLayout,
            "empty-cells" => Property::EmptyCells,
            "caption-side" => Property::CaptionSide,
            "vertical-align" => Property::VerticalAlign,

            // Content & Generation
            "content" => Property::Content,
            "quotes" => Property::Quotes,
            "counter-reset" => Property::CounterReset,
            "counter-increment" => Property::CounterIncrement,
            "content-visibility" => Property::ContentVisibility,
            "break-after" => Property::BreakAfter,
            "break-before" => Property::BreakBefore,
            "break-inside" => Property::BreakInside,

            // Misc
            "tab-size" => Property::TabSize,
            "visibility-collapse" => Property::VisibilityCollapse,
            "backface-visibility" => Property::BackfaceVisibility,
            "perspective" => Property::Perspective,
            "perspective-origin" => Property::PerspectiveOrigin,
            "transform" => Property::Transform,
            "transform-origin" => Property::TransformOrigin,
            "appearance" => Property::Appearance,
            "accent-color" => Property::AccentColor,
            "scroll-margin" => Property::ScrollMargin,
            "scroll-margin-top" => Property::ScrollMarginTop,
            "scroll-margin-right" => Property::ScrollMarginRight,
            "scroll-margin-bottom" => Property::ScrollMarginBottom,
            "scroll-margin-left" => Property::ScrollMarginLeft,
            "scroll-margin-block" => Property::ScrollMarginBlock,
            "scroll-margin-block-start" => Property::ScrollMarginBlockStart,
            "scroll-margin-block-end" => Property::ScrollMarginBlockEnd,
            "scroll-margin-inline" => Property::ScrollMarginInline,
            "scroll-margin-inline-start" => Property::ScrollMarginInlineStart,
            "scroll-margin-inline-end" => Property::ScrollMarginInlineEnd,
            "scroll-padding" => Property::ScrollPadding,
            "scroll-padding-top" => Property::ScrollPaddingTop,
            "scroll-padding-right" => Property::ScrollPaddingRight,
            "scroll-padding-bottom" => Property::ScrollPaddingBottom,
            "scroll-padding-left" => Property::ScrollPaddingLeft,
            "scroll-padding-block" => Property::ScrollPaddingBlock,
            "scroll-padding-block-start" => Property::ScrollPaddingBlockStart,
            "scroll-padding-block-end" => Property::ScrollPaddingBlockEnd,
            "scroll-padding-inline" => Property::ScrollPaddingInline,
            "scroll-padding-inline-start" => Property::ScrollPaddingInlineStart,
            "scroll-padding-inline-end" => Property::ScrollPaddingInlineEnd,

            _ => Property::Unknown(s.to_string()),
        }
    }

    /// Determines if a property is inherited by default according to CSS specifications.
    /// Returns true if the property is inherited, otherwise false.
    pub fn is_inherited(&self) -> bool {
        match self {
            // See: https://www.w3.org/TR/CSS21/propidx.html#initial-value
            // Typography
            Property::Color
            | Property::Font
            | Property::FontFamily
            | Property::FontSize
            | Property::FontStyle
            | Property::FontVariant
            | Property::FontWeight
            | Property::FontStretch
            | Property::FontFeatureSettings
            | Property::FontKerning
            | Property::FontOpticalSizing
            | Property::FontPalette
            | Property::FontSynthesis
            | Property::FontVariationSettings
            | Property::FontVariantCaps
            | Property::FontVariantLigatures
            | Property::FontVariantNumeric
            | Property::LineHeight
            | Property::LetterSpacing
            | Property::WordSpacing
            | Property::TextAlign
            | Property::TextIndent
            | Property::TextTransform
            | Property::TextDecoration
            | Property::TextDecorationColor
            | Property::TextDecorationStyle
            | Property::TextDecorationLine
            | Property::TextDecorationThickness
            | Property::TextShadow
            | Property::TextOverflow
            // Directionality
            | Property::Direction
            | Property::UnicodeBidi
            // List
            | Property::ListStyle
            | Property::ListStyleType
            | Property::ListStylePosition
            | Property::ListStyleImage
            // Misc
            | Property::Cursor
            | Property::Quotes
            | Property::Visibility
            | Property::WhiteSpace
            | Property::WordBreak
            | Property::OverflowWrap
            | Property::PointerEvents
            | Property::UserSelect
            | Property::CaretColor
            // SVG & other inherited CSS
            | Property::TabSize
            => true,
            Property::Unknown(_) => false,
            _ => false,
        }
    }
}
