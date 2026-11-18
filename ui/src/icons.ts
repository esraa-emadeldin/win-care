// Microsoft's Fluent UI System Icons (MIT), the icons Windows 11 itself uses.
// Imported as raw SVG so they inherit the text colour.
import home from '@fluentui/svg-icons/icons/home_24_regular.svg?raw'
import homeFilled from '@fluentui/svg-icons/icons/home_24_filled.svg?raw'
import apps from '@fluentui/svg-icons/icons/apps_24_regular.svg?raw'
import appsFilled from '@fluentui/svg-icons/icons/apps_24_filled.svg?raw'
import gauge from '@fluentui/svg-icons/icons/gauge_24_regular.svg?raw'
import gaugeFilled from '@fluentui/svg-icons/icons/gauge_24_filled.svg?raw'
import treemap from '@fluentui/svg-icons/icons/data_treemap_24_regular.svg?raw'
import treemapFilled from '@fluentui/svg-icons/icons/data_treemap_24_filled.svg?raw'
import settings from '@fluentui/svg-icons/icons/settings_24_regular.svg?raw'
import settingsFilled from '@fluentui/svg-icons/icons/settings_24_filled.svg?raw'
import info from '@fluentui/svg-icons/icons/info_24_regular.svg?raw'
import infoFilled from '@fluentui/svg-icons/icons/info_24_filled.svg?raw'
import search from '@fluentui/svg-icons/icons/search_20_regular.svg?raw'
import chevronDown from '@fluentui/svg-icons/icons/chevron_down_20_regular.svg?raw'
import chevronRight from '@fluentui/svg-icons/icons/chevron_right_20_regular.svg?raw'
import more from '@fluentui/svg-icons/icons/more_horizontal_20_regular.svg?raw'
import shield from '@fluentui/svg-icons/icons/shield_24_filled.svg?raw'
import trash from '@fluentui/svg-icons/icons/delete_20_regular.svg?raw'
import wand from '@fluentui/svg-icons/icons/wand_24_filled.svg?raw'
import play from '@fluentui/svg-icons/icons/play_20_filled.svg?raw'
import wifi from '@fluentui/svg-icons/icons/wifi_1_20_regular.svg?raw'
import bulb from '@fluentui/svg-icons/icons/lightbulb_24_filled.svg?raw'
import minimize from '@fluentui/svg-icons/icons/subtract_16_regular.svg?raw'
import maximize from '@fluentui/svg-icons/icons/square_16_regular.svg?raw'
import close from '@fluentui/svg-icons/icons/dismiss_16_regular.svg?raw'
import arrowRight from '@fluentui/svg-icons/icons/arrow_right_16_regular.svg?raw'

export const ICONS = {
  home,
  homeFilled,
  apps,
  appsFilled,
  gauge,
  gaugeFilled,
  treemap,
  treemapFilled,
  settings,
  settingsFilled,
  info,
  infoFilled,
  search,
  chevronDown,
  chevronRight,
  more,
  shield,
  trash,
  wand,
  play,
  wifi,
  bulb,
  minimize,
  maximize,
  close,
  arrowRight
}

export type IconName = keyof typeof ICONS
