//! Resources for the Wing shell.

mod animation;
mod content;
mod gesture;
mod shell;
mod theme;

/// Quick controls layout configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuickControlsLayout {
    /// Number of columns (3-6)
    pub columns: usize,
    /// Number of rows (auto-calculated based on content)
    pub rows: usize,
    /// Tile size in pixels
    pub tile_size: f32,
    /// Spacing between tiles
    pub tile_spacing: f32,
    /// Total height needed for the grid
    pub total_height: f32,
    /// Total number of tiles
    pub total_tiles: usize,
}

impl QuickControlsLayout {
    /// Target tile size (good touch target)
    const TILE_TARGET_SIZE: f32 = 64.0;
    /// Minimum tile size
    const TILE_MIN_SIZE: f32 = 48.0;
    /// Maximum tile size
    const TILE_MAX_SIZE: f32 = 80.0;
    /// Minimum columns
    const MIN_COLUMNS: usize = 3;
    /// Maximum columns
    const MAX_COLUMNS: usize = 6;
    /// Maximum rows (default limit)
    const MAX_ROWS: usize = 2;
    /// Tile spacing as ratio of tile size
    const TILE_SPACING_RATIO: f32 = 0.1; // 10% of tile size
    
    /// Compute layout based on available width.
    /// 
    /// Layout algorithm:
    /// 1. Calculate ideal columns based on width (四舍五入)
    /// 2. Calculate rows: ceil(total_tiles / columns)
    /// 3. Adjust if exceeds MAX_ROWS
    /// 4. Recalculate columns to fit
    /// 5. Adjust tile size to fill width exactly
    pub fn compute(available_width: f32, total_tiles: usize) -> Self {
        // Step 1: Calculate ideal columns (四舍五入)
        let ideal_columns = available_width / Self::TILE_TARGET_SIZE;
        let mut columns = (ideal_columns + 0.5) as usize;
        columns = columns.clamp(Self::MIN_COLUMNS, Self::MAX_COLUMNS);
        
        // Step 2: Calculate rows based on columns
        let mut rows = (total_tiles + columns - 1) / columns; // ceil division
        rows = rows.max(1);
        
        // Step 3: If rows exceeds MAX_ROWS, increase columns
        if rows > Self::MAX_ROWS {
            columns = (total_tiles + Self::MAX_ROWS - 1) / Self::MAX_ROWS;
            columns = columns.clamp(Self::MIN_COLUMNS, Self::MAX_COLUMNS);
            rows = Self::MAX_ROWS;
        }
        
        // Step 4: Calculate tile size to fill the available width
        let mut tile_size = Self::calc_tile_size(available_width, columns);
        
        // Step 5: Adjust if tile size is out of range
        // If tile is too large, try increasing columns (if we have room)
        if tile_size > Self::TILE_MAX_SIZE && columns < Self::MAX_COLUMNS {
            columns += 1;
            tile_size = Self::calc_tile_size(available_width, columns);
        }
        
        // If tile is still too large, cap it
        if tile_size > Self::TILE_MAX_SIZE {
            tile_size = Self::TILE_MAX_SIZE;
        }
        
        // If tile is too small, try decreasing columns
        if tile_size < Self::TILE_MIN_SIZE && columns > Self::MIN_COLUMNS {
            columns -= 1;
            tile_size = Self::calc_tile_size(available_width, columns);
        }
        
        // Recalculate rows after column adjustment
        rows = (total_tiles + columns - 1) / columns;
        rows = rows.max(1).min(Self::MAX_ROWS);
        
        Self::finalize(available_width, columns, rows, total_tiles)
    }
    
    fn finalize(available_width: f32, columns: usize, rows: usize, total_tiles: usize) -> Self {
        let tile_size = Self::calc_tile_size(available_width, columns);
        let tile_spacing = tile_size * Self::TILE_SPACING_RATIO;
        let total_height = tile_size * rows as f32 + tile_spacing * (rows.saturating_sub(1)) as f32;
        
        Self {
            columns,
            rows,
            tile_size,
            tile_spacing,
            total_height,
            total_tiles,
        }
    }
    
    /// Calculate tile size to fill available width with given columns.
    fn calc_tile_size(available_width: f32, columns: usize) -> f32 {
        let ratio = Self::TILE_SPACING_RATIO;
        available_width / (columns as f32 + (columns.saturating_sub(1)) as f32 * ratio)
    }
    
    /// Get tile position for given column and row index.
    /// Returns the center position of the tile.
    pub fn tile_position(&self, col: usize, row: usize, grid_start_x: f32, grid_start_y: f32) -> (f32, f32) {
        let x = grid_start_x + col as f32 * (self.tile_size + self.tile_spacing) + self.tile_size * 0.5;
        let y = grid_start_y + row as f32 * (self.tile_size + self.tile_spacing) + self.tile_size * 0.5;
        (x, y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellMetrics {
    pub screen_size: fhre::Vec2,
}

/// Proportional layout ratios (relative to screen dimensions).
/// These ratios are designed for mobile/watch-style shells.
impl ShellMetrics {
    pub const fn new(screen_size: fhre::Vec2) -> Self {
        Self { screen_size }
    }
    
    // === Screen dimensions ===
    
    #[inline]
    pub fn width(&self) -> f32 {
        self.screen_size.x
    }
    
    #[inline]
    pub fn height(&self) -> f32 {
        self.screen_size.y
    }
    
    #[inline]
    pub fn center_x(&self) -> f32 {
        self.screen_size.x * 0.5
    }
    
    #[inline]
    pub fn center_y(&self) -> f32 {
        self.screen_size.y * 0.5
    }
    
    // === Layout ratios ===
    
    /// Side inset for panels: 5% of screen width
    const SIDE_INSET_RATIO: f32 = 0.05;
    /// Notification card spacing: 3% of screen height
    const CARD_SPACING_RATIO: f32 = 0.03;
    /// Brightness bar height: 2% of screen height
    const BRIGHTNESS_HEIGHT_RATIO: f32 = 0.02;
    /// Text size ratio: 2.5% of screen height
    const TEXT_SIZE_RATIO: f32 = 0.025;
    /// Notification card height: 15% of screen height
    const NOTIFICATION_CARD_HEIGHT_RATIO: f32 = 0.15;
    /// Notification panel height: 65% of screen height
    const NOTIFICATION_PANEL_HEIGHT_RATIO: f32 = 0.65;
    /// Top padding for notification panel content: 2% of screen height
    const PANEL_TOP_PADDING_RATIO: f32 = 0.02;
    /// Spacing between sections in notification panel
    const SECTION_SPACING_RATIO: f32 = 0.02;
    
    // === Computed layout values ===
    
    #[inline]
    pub fn side_inset(&self) -> f32 {
        self.screen_size.x * Self::SIDE_INSET_RATIO
    }
    
    /// Compute quick controls layout based on screen width and tile count.
    pub fn quick_controls_layout(&self) -> QuickControlsLayout {
        let available_width = self.width() - self.side_inset() * 2.0;
        // 6 tiles: WiFi, Bluetooth, Airplane, Flashlight, DND, AutoRotate
        QuickControlsLayout::compute(available_width, 6)
    }
    
    /// Brightness bar height
    #[inline]
    pub fn brightness_height(&self) -> f32 {
        self.screen_size.y * Self::BRIGHTNESS_HEIGHT_RATIO
    }
    
    /// Section spacing in notification panel
    #[inline]
    pub fn section_spacing(&self) -> f32 {
        self.screen_size.y * Self::SECTION_SPACING_RATIO
    }
    
    /// Top padding for notification panel content
    #[inline]
    pub fn panel_top_padding(&self) -> f32 {
        self.screen_size.y * Self::PANEL_TOP_PADDING_RATIO
    }
    
    /// Quick controls section height (tiles only, no padding).
    pub fn quick_controls_tiles_height(&self) -> f32 {
        self.quick_controls_layout().total_height
    }
    
    /// Total height of header section (brightness + spacing + quick controls + spacing).
    pub fn notification_panel_header_height(&self) -> f32 {
        let top_padding = self.panel_top_padding();
        let brightness = self.brightness_height();
        let qc_tiles = self.quick_controls_tiles_height();
        let spacing = self.section_spacing();
        top_padding + brightness + spacing + qc_tiles + spacing
    }
    
    #[inline]
    pub fn notification_card_height(&self) -> f32 {
        self.screen_size.y * Self::NOTIFICATION_CARD_HEIGHT_RATIO
    }
    
    #[inline]
    pub fn notification_card_spacing(&self) -> f32 {
        self.screen_size.y * Self::CARD_SPACING_RATIO
    }
    
    #[inline]
    pub fn text_size(&self) -> f32 {
        self.screen_size.y * Self::TEXT_SIZE_RATIO
    }
    
    #[inline]
    pub fn text_size_small(&self) -> f32 {
        self.text_size() * 0.85
    }
    
    #[inline]
    pub fn notification_panel_height(&self) -> f32 {
        self.screen_size.y * Self::NOTIFICATION_PANEL_HEIGHT_RATIO
    }
    
    // === App surface ===
    
    #[inline]
    pub fn app_surface_width(&self) -> f32 {
        (self.screen_size.x * 0.60).min(self.screen_size.y * 0.40)
    }
    
    #[inline]
    pub fn app_surface_height(&self) -> f32 {
        (self.screen_size.y * 0.30).min(self.screen_size.y * 0.60)
    }
}

impl Default for ShellMetrics {
    fn default() -> Self {
        Self::new(fhre::Vec2::ZERO)
    }
}

impl fhre::resources::Resource for ShellMetrics {}

pub use animation::{ShellOverlayAnimation, ThemeAnimation};
pub use content::{
    ShellContent, ShellSurfaceEntry, ShellNotificationEntry, SurfaceState,
    NotificationCategory, NotificationPriority, QuickControlState,
};
pub use gesture::{GesturePhase, GestureState, SwipeDirection};
pub use shell::{ShellOverlayMode, ShellState};
pub use theme::{ThemeState, ThemeVariant};
