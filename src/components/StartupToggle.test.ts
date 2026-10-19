import { mount, flushPromises } from '@vue/test-utils'
import { expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import StartupToggle from './StartupToggle.vue'
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}))
it('打开设置只读取启动项，用户切换后才写入',async()=>{
  vi.mocked(invoke).mockResolvedValue(false)
  const view=mount(StartupToggle);await flushPromises()
  expect(invoke).toHaveBeenCalledWith('get_autostart')
  expect(invoke).not.toHaveBeenCalledWith('set_autostart',expect.anything())
  vi.mocked(invoke).mockResolvedValue(true)
  await view.find('input').setValue(true);await flushPromises()
  expect(invoke).toHaveBeenCalledWith('set_autostart',{enabled:true})
})
