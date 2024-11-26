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
  ]
}
```

EXAMPLES:

1. "I need an e-commerce platform for selling electronics"
- Entity: Product with name, SKU, price, specifications, stock level, brand, category, warranty info
- Entity: User with authentication, shipping addresses, payment methods, order history
- Entity: Order with status tracking, payment details, shipping details, order items
- Entity: Review with rating, comments, verified purchase status

2. "I'm building a healthcare appointment scheduling system"
- Entity: Patient with medical history, insurance info, contact details, emergency contacts
- Entity: Doctor with specialization, schedule, qualifications, license numbers
- Entity: Appointment with time slots, status, room number, visit type, notes
- Entity: MedicalRecord with diagnosis, prescriptions, test results, attachments

3. "I want to create a property management platform"
- Entity: Property with address, features, square footage, pricing, availability status
- Entity: Tenant with background check, rental history, payment records, documents
- Entity: MaintenanceRequest with priority level, description, status, photos
- Entity: Invoice with payment status, due dates, late fees, payment history

Think step-by-step:
1. What are the core data entities needed?
2. What fields would each entity require?
3. What features does each entity need?

Provide a complete, well-structured JSON response that captures these aspects. Be thorough but avoid overcomplicating the design.

REMEMBER:
- All entity names should be singular and PascalCase
- All field names should be camelCase
- Include standard fields like id, createdAt, updatedAt where appropriate
- Consider soft deletion if entities can be deleted
- Add search capabilities for entities that might need filtering
- Consider authentication needs for sensitive entities

Based on the above requirements, please provide a detailed analysis in the specified JSON format.
"#;
