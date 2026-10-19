// Mechanical localization of static Vue text nodes; dynamic IDs remain stable.
import fs from 'node:fs'
import { parse } from '@vue/compiler-sfc'
import { parse as baseParse } from '@vue/compiler-dom'
const files = ['src/App.vue', ...fs.readdirSync('src/components').filter(name=>name.endsWith('.vue')).map(name=>`src/components/${name}`)]
for (const file of files) {
  let source=fs.readFileSync(file,'utf8')
  if(source.includes("import { t } from")) continue
  const block=parse(source).descriptor.template
  if(!block) continue
  const edits=[]
  function walk(node) {
    if(node.type===2 && /[\u3400-\u9fff]/.test(node.content)) {
      const text=node.content.trim()
      if(text) edits.push({start:node.loc.start.offset,end:node.loc.end.offset,text:`{{ t(${JSON.stringify(text)}) }}`})
    }
    if(node.props) for(const prop of node.props) {
      if(prop.type===6 && ['aria-label','placeholder','title'].includes(prop.name) && prop.value && /[\u3400-\u9fff]/.test(prop.value.content)) edits.push({start:prop.loc.start.offset,end:prop.loc.end.offset,text:`:${prop.name}='t(${JSON.stringify(prop.value.content).replaceAll("'","&#39;")})'`})
    }
    if(node.children) for(const child of node.children)walk(child)
  }
  walk(baseParse(block.content))
  let template=block.content
  for(const edit of edits.sort((a,b)=>b.start-a.start))template=template.slice(0,edit.start)+edit.text+template.slice(edit.end)
  source=source.slice(0,block.loc.start.offset)+template+source.slice(block.loc.end.offset)
  const importPath=file==='src/App.vue'?'./i18n':'../i18n'
  source=source.replace('<script setup lang="ts">',`<script setup lang="ts">\nimport { t } from '${importPath}'`)
  fs.writeFileSync(file,source)
}
