pub const PROMPT: &str = r#"USER REQUIREMENTS:
{user_prompt}

Please analyze the requirements and provide a detailed response in the following JSON structure:

```json
{
  "entities": [
    {
      "name": "string",         // PascalCase entity name
      "description": "string",  // Brief description of the entity's purpose
      "fields": [              // Core fields this entity should have
        {
          "name": "string",    // camelCase field name
          "type": "string",    // TypeScript type
          "description": "string",
          "required": boolean
        }
      ],
      "features": {            // Entity-specific features needed
        "hasAuth": boolean,    // Whether this entity needs authentication
        "needsSearch": boolean,
        "needsSoftDelete": boolean,
        "hasTimestamps": boolean
      }
    }
  ],
  "relationships": [
    {
      "from": "string",        // Source entity name
      "to": "string",          // Target entity name
      "type": "ONE_TO_ONE" | "ONE_TO_MANY" | "MANY_TO_MANY",
      "description": "string"  // Description of the relationship
    }
  ],
  "projectFeatures": {
    "auth": {
      "type": "JWT" | "SESSION",
      "magicLink": boolean,
      "password": boolean,
      "oauth": boolean,
    },
    "database": {
      "service": "MONGO" | "POSTGRES" | "SUPABASE" | "FIREBASE",
    }
    "email": {
      "service": "RESEND" | "SENDGRID" | "MAILGUN",
    },
  }
}
```

EXAMPLES:

1. "I want an app for users who can own recipes and share them with others"
- Entities: User, Recipe
- Authentication: Required (for user management)
- Relationships: User owns many Recipes, Users can share Recipes
- Security: Recipe access control, User data protection

2. "I need a task management system where team leaders can assign tasks to team members"
- Entities: User, Team, Task
- Authentication: Required with roles (team leader, team member)
- Relationships: Team has many Users, User owns many Tasks, Tasks assigned to Users
- Security: Role-based access control, Task visibility rules

3. "I want to create a blog platform where authors can publish articles and readers can leave comments"
- Entities: User, Article, Comment
- Authentication: Required (authors vs readers)
- Relationships: User writes many Articles, Articles have many Comments
- Security: Content moderation, Author verification

4. "I need an e-commerce platform for selling digital products with user reviews"
- Entities: User, Product, Order, Review
- Authentication: Required with payment integration
- Relationships: User places Orders, Orders contain Products, Products have Reviews
- Security: Payment data protection, Purchase verification

Think step-by-step:
1. What are the core data entities needed?
2. What fields would each entity require?
3. How do these entities relate to each other?
4. What security measures are needed to protect the data?
5. What additional features are implied by the requirements?

Provide a complete, well-structured JSON response that captures all these aspects. Be thorough but avoid overcomplicating the design.

REMEMBER:
- All entity names should be singular and PascalCase
- All field names should be camelCase
- Include standard fields like id, createdAt, updatedAt where appropriate
- Consider soft deletion if entities can be deleted
- Add search capabilities for entities that might need filtering
- Consider audit trails for sensitive operations

Based on the above requirements, please provide a detailed analysis in the specified JSON format.
"#;
