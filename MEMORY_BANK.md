# Memory Bank: בניית Document DB ב-Rust

מסמך זה מתעד את ההתקדמות, המושגים שנלמדו, וההחלטות העיצוביות שהתקבלו במהלך בניית מסד נתונים מבוסס מסמכים ב-Rust.

## החלטות ארכיטקטוניות

*   **`DataBase` -> `Collection`**: הוחלט לשנות את שם מבנה הנתונים הראשי מ-`DataBase` ל-`Collection`. זה משקף בצורה מדויקת יותר את המודל המקובל במסדי נתונים מבוססי מסמכים.
*   **פישוט API**: הוסרה הפונקציה `from_hashmap` והפונקציה `new` פושטה, בהתאם לעיקרון YAGNI (You Ain't Gonna Need It) כדי לשמור על הקוד נקי וממוקד.

## מושגי מפתח שנלמדו ויושמו בהצלחה

1.  **ניהול פרויקט עם Cargo**:
    *   יצירת פרויקט חדש (`cargo new`).
    *   קומפילציה והרצה (`cargo run`).
    *   שימוש ב-`.gitignore` סטנדרטי לפרויקטי Rust.

2.  **בדיקות יחידה (Unit Testing)**:
    *   העברת כל לוגיקת הבדיקה מפונקציית `main` למודול בדיקות ייעודי (`#[cfg(test)] mod tests`).
    *   הרצת בדיקות באמצעות `cargo test`.
    *   שימוש במאקרואים `assert_eq!` ו-`assert!` לאימות התנהגות נכונה.
    *   כתיבת בדיקות למקרי קצה ושגיאות (למשל, פעולה על מפתח לא קיים).
    *   שימוש במאקרו `matches!` לבדיקת וריאנטים ספציפיים של `enum` בתוצאות שגיאה.
    *   שימוש בבדיקות לאיתור ותיקון באגים (Red-Green-Refactor).

3.  **מבני נתונים ו-API**:
    *   הגדרת `struct` ו-`enum`.
    *   שימוש ב-`HashMap` ומתודות הליבה שלו (`get`, `get_mut`, `insert`, `remove`).

4.  **בעלות והשאלות (Ownership & Borrowing)**:
    *   הבנה והבחנה ברורה בין סוגי `self` (`&self`, `&mut self`).
    *   פתרון שגיאת קומפילציה קלאסית (E0502) של השאלה כפולה (immutable vs. mutable borrow) באמצעות `.clone()` לשחרור השאלה.
    *   הבנת השימוש ב-`*` (dereference) לעדכון ערך דרך רפרנס משתנה.

5.  **טיפול בתוצאות ושגיאות**:
    *   שימוש ב-`Option<T>`, `Result<T, E>`, והתבנית `Result<Option<T>, E>`.
    *   שימוש בטוח ב-`match` ו-`if let` לטיפול בתוצאות.

## API הליבה שפותח (CRUD)

פותח API מלא ונקי עבור `Collection`, המכוסה במלואו על ידי בדיקות יחידה:

*   **Create**: `fn write(&mut self, value: String) -> Result<u64, OperationError>`
*   **Read**: `fn read(&self, key: u64) -> Result<Option<&String>, OperationError>`
*   **Update**: `fn update(&mut self, key: u64, new_value: &String) -> Result<u64, OperationError>`
*   **Delete**: `fn delete(&mut self, key: u64) -> Result<String, OperationError>`
