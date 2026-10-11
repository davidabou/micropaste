// Classes of the context menus, shared by the tabs and the tiles. Colors of the Windows 11 menus

export const menuContent = 'z-50 min-w-44 origin-(--reka-context-menu-content-transform-origin) rounded-lg border border-black/[5.78%] bg-[#f9f9f9] p-1 font-ui text-xs shadow-lg select-none dark:border-black/20 dark:bg-[#2c2c2c]'

/** Icon in front of a menu item label */
export const menuIcon = 'shrink-0 text-sm opacity-70'

export const menuItem = 'flex h-6 cursor-default items-center gap-2 rounded-[4px] px-2 outline-none data-disabled:opacity-40 data-highlighted:bg-black/[3.73%] dark:data-highlighted:bg-white/[6.05%]'

/** Motion props of a menu: pops in from where it opens on a bouncy spring, pops out without bounce */
export const menuMotion = {
  initial: { opacity: 0, scale: 0.85 },
  animate: { opacity: 1, scale: 1, transition: { type: 'spring' as const, bounce: 0.35, visualDuration: 0.2 } },
  exit: { opacity: 0, scale: 0.9, transition: { type: 'spring' as const, bounce: 0, visualDuration: 0.1 } }
}
