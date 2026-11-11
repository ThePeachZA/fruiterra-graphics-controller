# Teams and Talent

Teams hold the identity, colours and roster that the scoreboard and roster
overlays display. Talent holds the people shown by the casters overlay and the
interview desk.

## Teams

Open **Teams** in the sidebar. The left panel lists every team and shows its
tag, name, colour and player count.

### Create a team

1. Select **+ New team**.
2. The new team is selected automatically and opens on **Team Info**.

### Team Info

| Field | What it does |
|---|---|
| **Team name** | The full name shown on the overlays. |
| **Team tag** | The short tag, usually 3–4 letters. |
| **Record** | Free text such as `12-3`, shown on the scoreboard. |
| **Team logo** | An image from the **Logo** category in Assets. |
| **Background** | The team's primary colour. |
| **Text** | The colour used for text on the team's background. |
| **Logo background** | The colour behind the logo. |

The **Preview** panel shows the badge and the scoreboard treatment as you edit.
Changes save automatically.

### Players

Open the **Players** tab to see the ordered roster.

1. Select **+ Add player**.
2. Fill in the editor:
   - **Player name**
   - **Role** — Tank, Damage or Support
   - **Position** — Core, Sub, Manager or Coach
   - **Portrait** — an image from the **Portrait** category
   - **Featured hero** — an image from the **Hero** category
   - **Pronouns**
3. Select **Done** to close the editor, or **Delete player** to remove them.

Core players, substitutes, managers and coaches share one roster. The order in
the saved project is the order the roster overlays use.

### Delete a team

Select **Delete team** in the team header.

- If nothing uses the team, the app asks for confirmation and deletes it.
- If the team is still used, the dialog lists every tournament and current-match
  reference. You can cancel, or select **Remove references and delete** to clear
  those references and delete the team.

Deletion cannot be undone.

## Talent

Open **Talent** in the sidebar. The left panel lists every talent member with
their name, pronouns and social platform.

### Add or edit talent

1. Select **+ Add talent**.
2. Fill in:
   - **Display name**
   - **Real name** (optional)
   - **Pronouns**
   - **Social platform** — Twitter / X, Twitch, YouTube, Instagram or TikTok
   - **Handle** — for example `@name`
   - **Portrait** — an image from the **Portrait** category
3. Changes save automatically.

Select **Delete talent** to remove the person. Any current-match caster, panel or
interviewer assignment that used them is cleared.

## Use talent in a match

Open **Match Controller → Talent** and assign:

- **Caster 1** and **Caster 2**
- **Panel 1**, **Panel 2** and **Panel 3**
- **Interviewer**

The interview desk also has an **Interviewee** picker. If no talent member fits,
leave **Interviewee** empty and use **Custom name** and **Custom subtext**
instead.

## Related pages

- [Assets](Assets)
- [Match Controller](Match-Controller)
- [Overlays in OBS](Overlays-in-OBS)
