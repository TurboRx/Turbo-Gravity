# Contributing to Turbo Gravity

Thank you for your interest in contributing to Turbo Gravity! We welcome contributions from the community and are excited to see what you'll bring to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Code Style Guidelines](#code-style-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Enhancements](#suggesting-enhancements)

## Code of Conduct

This project and everyone participating in it is expected to uphold a respectful and welcoming environment. Please be kind and courteous to others.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the [issue tracker](https://github.com/TurboRx/Turbo-Gravity/issues) to see if the problem has already been reported. If it has and the issue is still open, add a comment to the existing issue instead of opening a new one.

When creating a bug report, please include:

- **A clear and descriptive title**
- **Steps to reproduce the behavior**
- **Expected behavior**
- **Actual behavior**
- **Screenshots** (if applicable)
- **Environment details** (Node.js version, OS, etc.)
- **Additional context** (error messages, logs, etc.)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **A clear and descriptive title**
- **A detailed description** of the proposed enhancement
- **Use cases** explaining why this enhancement would be useful
- **Possible implementation** (if you have ideas)

### Pull Requests

We actively welcome your pull requests! Here's how to contribute code:

1. Fork the repository and create your branch from `main`
2. Make your changes following our code style guidelines
3. Test your changes thoroughly
4. Update documentation if needed
5. Submit a pull request

## Getting Started

### Prerequisites

Before you begin, ensure you have:

- Node.js 18 or higher
- npm (comes with Node.js)
- MongoDB (local installation or hosted service)
- A Discord Application (create one at [Discord Developer Portal](https://discord.com/developers/applications))
- Git

### Setting Up Your Development Environment

1. **Fork and Clone**
   ```bash
   git clone https://github.com/YOUR_USERNAME/Turbo-Gravity.git
   cd Turbo-Gravity
   ```

2. **Install Dependencies**
   ```bash
   npm install
   ```

3. **Configure the Application**
   ```bash
   npm start
   ```
   Then navigate to `http://localhost:8080/setup` and configure your bot through the web interface.

4. **Start Development**
   ```bash
   npm run dev
   ```
   This uses nodemon to auto-restart on file changes.

### Project Structure

```
Turbo-Gravity/
├── src/
│   ├── commands/        # Discord bot commands
│   ├── dashboard/       # Web dashboard routes and views
│   ├── models/          # MongoDB models
│   ├── BotManager.js    # Bot lifecycle management
│   └── localConfig.js   # Configuration handling
├── index.js             # Application entry point
├── package.json         # Dependencies and scripts
└── Dockerfile           # Docker configuration
```

## Development Process

### Branching Strategy

- `main` - Production-ready code
- `feature/*` - New features
- `bugfix/*` - Bug fixes
- `docs/*` - Documentation changes

### Making Changes

1. **Create a branch** for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** in small, logical commits

3. **Test your changes** thoroughly:
   - Run the application and test affected functionality
   - Test the web dashboard if you modified UI/routes
   - Test bot commands if you modified command files

4. **Keep your branch updated** with main:
   ```bash
   git fetch origin
   git rebase origin/main
   ```

## Code Style Guidelines

### JavaScript/Node.js

- Use **ES6+ syntax** (import/export, arrow functions, etc.)
- Use **async/await** over promises where possible
- Use **2 spaces** for indentation
- Use **single quotes** for strings
- Use **semicolons** at the end of statements
- Use **descriptive variable names** (no single-letter variables except in loops)
- Add **JSDoc comments** for functions and classes

### Discord Commands

When creating new Discord commands:

```javascript
import { SlashCommandBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('commandname')
    .setDescription('Clear description of what this command does'),
  
  async execute(interaction) {
    // Command logic here
    await interaction.reply('Response');
  }
};
```

- Place commands in the appropriate category folder under `src/commands/`
- Use clear, concise command names and descriptions
- Handle errors gracefully with try/catch blocks
- Respond to interactions within 3 seconds (use `deferReply` for longer operations)

### Dashboard Routes

- Follow RESTful conventions for routes
- Validate user input and sanitize data
- Use proper HTTP status codes
- Include error handling middleware
- Protect admin routes with authentication checks

## Commit Message Guidelines

Write clear and meaningful commit messages:

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less
- Reference issues and pull requests when applicable

Examples:
```
Add user profile command
Fix dashboard crash on invalid session
Update README with Docker instructions
Refactor authentication middleware
```

## Pull Request Process

1. **Update Documentation**
   - Update the README.md if you changed functionality
   - Add JSDoc comments to new functions
   - Update relevant documentation files

2. **Ensure Your PR**
   - Has a clear title describing the change
   - Includes a description of what changed and why
   - References any related issues (e.g., "Fixes #123")
   - Has been tested and works as expected
   - Follows the code style guidelines

3. **PR Template**
   ```markdown
   ## Description
   Brief description of changes
   
   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update
   
   ## Testing
   How did you test these changes?
   
   ## Related Issues
   Fixes #(issue)
   ```

4. **Review Process**
   - Maintainers will review your PR
   - Address any feedback or requested changes
   - Once approved, your PR will be merged

## Reporting Bugs

Found a bug? Please create an issue with:

- **Title**: Short, descriptive title
- **Description**: Detailed description of the issue
- **Steps to Reproduce**: Step-by-step instructions
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Environment**: OS, Node.js version, etc.
- **Logs/Screenshots**: Any relevant error messages or screenshots

## Suggesting Enhancements

Have an idea? Create an enhancement issue with:

- **Title**: Clear feature request title
- **Description**: Detailed description of the enhancement
- **Motivation**: Why this feature would be useful
- **Implementation Ideas**: Your thoughts on implementation (optional)

---

## Questions?

If you have questions about contributing, feel free to:

- Open an issue with the `question` label
- Join our community discussions
- Reach out to the maintainers

## Recognition

Contributors will be recognized in our release notes and project documentation. Thank you for making Turbo Gravity better!

---

## License

By contributing to Turbo Gravity, you agree that your contributions will be licensed under the [MIT License](LICENSE).
