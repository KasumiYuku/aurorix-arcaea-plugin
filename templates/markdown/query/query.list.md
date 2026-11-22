## 成绩清单 · {{ name }}

```
收录 {{ known }} 张 · 命中 {{ matched }} 条
{{ head }}
```

> {{ note }}

---

{{#each rows}}{{ index }}. **{{ title }}**　{{ difficulty }} {{ constant }}　{{ score }}　LS {{ loss }}　{{ grade }}
{{/each}}