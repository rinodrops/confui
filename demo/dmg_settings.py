import os.path

# Application bundle
application = defines.get('app', 'dist/Settings.app')
appname = os.path.basename(application)

# Contents of the DMG
files = [application]
symlinks = {'Applications': '/Applications'}

# Icon positions (logical points, origin = top-left)
icon_locations = {
    appname:        (190, 190),
    'Applications': (470, 190),
}

# Background image (1320x800 @2x for Retina, logical window 660x400).
# dmgbuild loads this file via exec(), so __file__ is unavailable; pass demo_dir
# from the Justfile (-D demo_dir=...).
_demo_dir = defines.get('demo_dir', 'demo')
_bg = os.path.join(_demo_dir, 'assets', 'dmg-background.png')
background = _bg if os.path.isfile(_bg) else None

# Finder window appearance
show_status_bar  = False
show_tab_view    = False
show_toolbar     = False
show_pathbar     = False
show_sidebar     = False
sidebar_width    = 180

# Window rect: ((x, y from bottom-left of screen), (width, height))  -- screen coords only
window_rect = ((200, 400), (660, 430))

default_view      = 'icon-view'
show_icon_preview = False
icon_size         = 128
text_size         = 13
