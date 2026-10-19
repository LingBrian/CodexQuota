import { mount, flushPromises } from '@vue/test-utils'
import { beforeEach, expect, it, vi } from 'vitest'
import OverlayEditor from './OverlayEditor.vue'
import type { AppSettings } from '../types'
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn().mockResolvedValue([])}))
let settings:AppSettings
beforeEach(()=>{settings={refreshSeconds:60,overlayVisible:true,overlayTextColor:'#ffffff',overlayFontSize:14,overlayOpacity:1,overlay:{showFive:true,showSeven:false,reverse:false,fontWeight:600,background:'#000000',backgroundOpacity:1,gap:16,edgeOffset:9,monitor:2,width:300,height:46,clickThrough:true,showCountdown:false,showUsed:false,progress:false,lowColor:'#ff0000'}}})
it('预设立即保存样式，但不重置显示器、偏移或穿透偏好',async()=>{
  const view=mount(OverlayEditor,{props:{value:settings}});await flushPromises()
  await view.find('[data-preset="bars"]').trigger('click')
  expect(settings.overlay?.progress).toBe(true)
  expect(settings.overlay?.showSeven).toBe(true)
  expect(settings.overlay?.height).toBe(48)
  expect(settings.overlay?.monitor).toBe(2)
  expect(settings.overlay?.edgeOffset).toBe(9)
  expect(settings.overlay?.clickThrough).toBe(true)
  expect(settings.overlay?.background).toBe('#1b1e23')
  expect(settings.overlay?.width).toBe(420)
  expect(settings.overlay?.displayStyle).toBe('bars')
  expect(settings.overlay?.cornerRadius).toBe(8)
  expect(view.emitted('save')).toHaveLength(1)
  await view.find('[data-preset="text"]').trigger('click')
  expect(settings.overlay?.backgroundOpacity).toBe(0)
  expect(settings.overlay?.progress).toBe(true)
})
it('亮色与暗色透明预设更新配色和透明度但保留显示器与位置偏好',async()=>{
  const view=mount(OverlayEditor,{props:{value:settings}});await flushPromises()
  await view.find('[data-preset="light-glass"]').trigger('click')
  expect(settings.overlay?.background).toBe('#edf4f8')
  expect(settings.overlay?.backgroundOpacity).toBe(0.78)
  expect(settings.overlayTextColor).toBe('#17212b')
  expect(settings.overlay?.monitor).toBe(2)
  expect(settings.overlay?.edgeOffset).toBe(9)
  await view.find('[data-preset="dark-glass"]').trigger('click')
  expect(settings.overlay?.background).toBe('#111923')
  expect(settings.overlayTextColor).toBe('#f4f7fb')
})
it('七种预设分别套用外观且保留任务栏交互偏好',async()=>{
  const cases=[
    ['capsule','#19221f',0.96,'#f2fff9',22],['bars','#1b1e23',0.94,'#f4f7fb',8],['text','#111827',0,'#ffffff',0],
    ['light-glass','#edf4f8',0.78,'#17212b',16],['dark-glass','#111923',0.72,'#f4f7fb',16],
    ['light-solid','#f8fafc',1,'#202a35',9],['dark-solid','#171a20',1,'#f4f7fb',9],
  ] as const
  for (const [id,background,alpha,textColor,radius] of cases) {
    settings={...settings,overlay:{...settings.overlay!,monitor:2,edgeOffset:9,clickThrough:true}}
    const view=mount(OverlayEditor,{props:{value:settings}});await flushPromises()
    await view.find(`[data-preset="${id}"]`).trigger('click')
    expect(settings.overlay?.displayStyle).toBe(id)
    expect(settings.overlay?.background).toBe(background)
    expect(settings.overlay?.backgroundOpacity).toBe(alpha)
    expect(settings.overlay?.cornerRadius).toBe(radius)
    expect(settings.overlay?.progress).toBe(true)
    expect(settings.overlayTextColor).toBe(textColor)
    expect(settings.overlay?.monitor).toBe(2)
    expect(settings.overlay?.edgeOffset).toBe(9)
    expect(settings.overlay?.clickThrough).toBe(true)
    view.unmount()
  }
})
it('两个指标可独立开关并提交保存事件',async()=>{
  const view=mount(OverlayEditor,{props:{value:settings}});await flushPromises()
  const label=view.findAll('label').find(item=>item.text()==='显示 5h')!
  await label.find('input').setValue(false)
  expect(settings.overlay?.showFive).toBe(false)
  expect(view.emitted('save')?.length).toBeGreaterThan(0)
})
it('沿栏位置与桌面横纵坐标均提交实际修改',async()=>{
  settings.overlay!.placement='taskbar'
  const view=mount(OverlayEditor,{props:{value:settings}});await flushPromises()
  await view.find('[data-taskbar-offset]').setValue(240)
  expect(settings.overlay!.edgeOffset).toBe(240)
  await view.find('[data-placement]').setValue('desktop')
  // The parent normally supplies reactive settings.
  await view.setProps({value:{...settings,overlay:{...settings.overlay!}}})
  await view.find('[data-offset-x]').setValue(350)
  await view.find('[data-offset-y]').setValue(180)
  expect(view.props('value').overlay?.offsetX).toBe(350)
  expect(view.props('value').overlay?.offsetY).toBe(180)
  expect(view.emitted('save')!.length).toBeGreaterThanOrEqual(4)
})
