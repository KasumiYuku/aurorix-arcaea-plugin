## Best 50 · {{ name }}

```
当前 PTT　{{ ptt }}
Best 50　 {{ best50 }}
Best 10　 {{ best10 }}
Max PTT　 {{ max_ptt }}
参与计算　{{ count }} 首
```

> {{ freshness }}
{{#each notes}}> {{ text }}
{{/each}}

---

{{#each records}}{{ index }}. **{{ title }}**　{{ grade }}　{{ difficulty }} {{ constant }}
　　{{ score }}　PTT {{ rating }}　P/{{ judge }}
{{/each}}