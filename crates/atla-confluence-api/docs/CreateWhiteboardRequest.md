# CreateWhiteboardRequest

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**space_id** | **String** | ID of the space. |
**title** | Option<**String**> | Title of the whiteboard. | [optional]
**parent_id** | Option<**String**> | The parent content ID of the whiteboard. | [optional]
**template_key** | Option<**TemplateKey**> | Providing a template key will add that template to the new whiteboard. (enum: 2x2-prioritization, 4ls-retro, annual-calendar, brainwriting, concept-map, crazy-8s, daily-sync, disruptive-brainstorm, dot-voting, elevator-pitch, flow-chart, gap-analysis, ice-breakers, incident-postmortem, journey-mapping-kit, kanban-board, lean-coffee, network-of-teams, org-chart, pi-planning, prioritization, prioritization-experiment, product-roadmap, product-vision-board, rice, sailboat-retro, service-blueprint, simple-retrospective, sprint-planning, sticky-note-pack, swimlanes, team-formation-guide, timeline, timeline-workflow, user-story-map, workflow, vision-board, venn-diagram, storyboard, action-plan, root-cause-analysis, executive-summary, stakeholder-mapping, annual-calendar-2025-2026, health-monitor, okr-planning, swot-analysis, poker-planning, fishbone-diagram, risk-assessment, bounded-context, hopes-and-fears, swimlane-vertical) | [optional]
**locale** | Option<**Locale**> | If templateKey is provided, locale will decide which language the template will be created with. If locale is omitted, the user's locale will be used. (enum: de-DE, cs-CZ, ko-KR, fr-FR, it-IT, ja-JP, nl-NL, nb-NO, da-DK, sv-SE, fi-FI, ru-RU, pl-PL, tr-TR, hu-HU, en-GB, en-US, pt-BR, zh-CN, zh-TW, es-ES) | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)
