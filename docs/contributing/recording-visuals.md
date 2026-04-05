# Recording Visual Evidence

This guide covers tools and workflows for creating GIFs and screenshots for PRs and documentation.

## Quick Reference

| Tool | Type | Platform | Best For |
|------|------|----------|----------|
| **Kap** | Screen recorder | macOS | Quick UI demos, free |
| **CleanShot X** | Screen recorder | macOS | Professional polished demos |
| **vhs** | Terminal recorder | All | CLI command demos |
| **asciinema** | Terminal recorder | All | Shareable terminal sessions |
| **LICEcap** | Screen recorder | Win/Mac | Simple lightweight GIFs |

## Recommended Workflow

### For UI Features

1. **Prepare the scene**
   - Use consistent window size: 1200x800 or 1440x900
   - Clear browser cache/disable extensions that clutter UI
   - Use browser DevTools device toolbar for consistent sizing

2. **Record with Kap**
   ```bash
   brew install kap
   ```
   - Select area/window
   - Record (keep under 10 seconds)
   - Export as GIF (fps: 15, quality: high)

3. **Save to project**
   ```bash
   mv ~/Downloads/recording.gif docs/assets/gifs/feature-name.gif
   ```

### For CLI Features

1. **Record with vhs**
   ```bash
   brew install charmbracelet/tap/vhs
   ```

2. **Create tape file**
   ```bash
   cat > demo.tape << 'EOF'
   Output docs/assets/gifs/feature-cli.gif
   
   Set FontSize 14
   Set Width 1200
   Set Height 600
   
   Type "agileplus status"
   Sleep 500ms
   Enter
   Sleep 2s
   
   Type "agileplus list"
   Sleep 500ms
   Enter
   Sleep 2s
   EOF
   ```

3. **Generate GIF**
   ```bash
   vhs < demo.tape
   ```

### For API/Backend

1. Use Insomnia/Postman
2. Run request
3. Screenshot response with metadata
4. Annotate with arrows/circles if needed

## Naming Conventions

```
docs/assets/gifs/
├── <feature-id>-demo.gif          # Main feature demo
├── <feature-id>-usage.gif          # Usage walkthrough
├── <feature-id>-troubleshoot.gif   # Common issues

# Example:
docs/assets/gifs/
├── 018-service-controls-demo.gif
├── 018-service-controls-usage.gif
├── 023-pr-requirements-demo.gif
```

## Optimization

### Reduce GIF Size

```bash
# Using gifsicle (brew install gifsicle)
gifsicle -O3 --colors 128 input.gif -o output.gif

# Using ffmpeg (for large recordings)
ffmpeg -i input.mov -vf "fps=15,scale=1200:-1:flags=lanczos,split[s0][s1];[s0]palettegen=[s1]paletteuse" output.gif
```

### Guidelines

- **Duration**: Keep under 10 seconds (ideally 5-7)
- **File size**: Under 2MB for GitHub
- **Resolution**: 1200px width max
- **FPS**: 15-20 is sufficient
- **Focus**: Show one action per GIF

## Embedding in PRs

### GitHub PR Description

```markdown
## Visual Evidence

![feature-demo](docs/assets/gifs/018-service-controls.gif)

*Service control panel with start/stop/restart functionality*
```

### Documentation

```markdown
# Feature Name

![Feature demo](../../assets/gifs/feature-demo.gif)

## Overview
...
```

### Changelog

```markdown
- **scope**: Add feature (#123)
  ![feature](docs/assets/gifs/feature.gif)
  [Documentation →](docs/path/to/feature.md)
```

## Git LFS (Optional)

For large GIFs, use Git LFS:

```bash
# Track GIFs with LFS
git lfs track "docs/assets/gifs/*.gif"
git add .gitattributes
```

## Checklist Before Recording

- [ ] Close unrelated apps/tabs
- [ ] Use consistent browser window size
- [ ] Clear notifications
- [ ] Test recording (5 second test first)
- [ ] Check file size under 2MB
- [ ] Verify GIF loops smoothly

## Related

- [Documentation Guide](./documentation.md)
- [PR Requirements Policy](../../GOVERNANCE_PR_REQUIREMENTS.md)
