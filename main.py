import os

import streamlit as st

from python_src.StylistAPIComponent import Gender, StylistAPIComponent

st.set_page_config(page_title="Fashion Similarity Finder", layout="wide")

API_BASE_URL = "http://localhost:9500"  # Adjust if needed
api = StylistAPIComponent(API_BASE_URL)

def main():
    st.title("Fashion Similarity Finder")

    st.sidebar.header("Controls")
    page = st.sidebar.radio("Select Page", ["Upload Clothes", "View Clothes", "Find Similar"])

    if page == "Upload Clothes":
        upload_clothes_page()
    elif page == "View Clothes":
        view_clothes_page()
    elif page == "Find Similar":
        find_similar_page()

def upload_clothes_page():
    st.header("Upload Clothes")
    upload_mode = st.radio("Upload Mode", ["Single", "Batch"])

    if upload_mode == "Single":
        # Single upload form
        with st.form("single_upload_form"):
            name = st.text_input("Clothing Name")
            gender = st.selectbox("Gender", [Gender.MALE.value, Gender.FEMALE.value])
            uploaded_file = st.file_uploader("Choose an image", type=['png', 'jpg', 'jpeg'])

            submit = st.form_submit_button("Upload")
            if submit and uploaded_file and name:
                temp_path = f"temp_{uploaded_file.name}"
                with open(temp_path, "wb") as f:
                    f.write(uploaded_file.getbuffer())

                try:
                    api.upload_clothes(name, Gender(gender), temp_path)
                    st.success("Successfully uploaded clothing item!")
                    st.image(uploaded_file, caption=name, width=300)
                except Exception as e:
                    st.error(f"Error uploading: {str(e)}")
                finally:
                    if os.path.exists(temp_path):
                        os.remove(temp_path)

    else:
        # Batch upload mode
        uploaded_files = st.file_uploader("Choose images", type=['png', 'jpg', 'jpeg'], accept_multiple_files=True)
        if uploaded_files:
            with st.form("batch_upload_form"):
                st.write("Provide details for each item:")
                items = []
                for idx, file in enumerate(uploaded_files):
                    st.write(f"**Item {idx+1}: {file.name}**")
                    name = st.text_input("Clothing Name", key=f"name_{idx}")
                    gender = st.selectbox("Gender", [Gender.MALE.value, Gender.FEMALE.value], key=f"gender_{idx}")
                    items.append((file, name, gender))
                
                submit_all = st.form_submit_button("Upload All")
                if submit_all:
                    all_uploaded = True
                    for file, n, g in items:
                        if file and n:
                            temp_path = f"temp_{file.name}"
                            with open(temp_path, "wb") as f:
                                f.write(file.getbuffer())
                            try:
                                api.upload_clothes(n, Gender(g), temp_path)
                            except Exception as e:
                                st.error(f"Error uploading {n}: {str(e)}")
                                all_uploaded = False
                            finally:
                                if os.path.exists(temp_path):
                                    os.remove(temp_path)
                    if all_uploaded:
                        st.success("All items uploaded successfully!")

def view_clothes_page():
    st.header("View Clothes")
    try:
        clothes = api.get_clothes()
        if not clothes:
            st.info("No clothes found in the database.")
            return

        cols = st.columns(3)
        for idx, item in enumerate(clothes):
            with cols[idx % 3]:
                st.subheader(item['name'])
                st.write(f"ID: {item['id']}")
                st.write(f"Descriptions: {item['descriptions']}")
                
                image_data = item.get('image', '')
                if image_data:
                    import base64
                    from io import BytesIO
                    image_bytes = base64.b64decode(image_data)
                    st.image(BytesIO(image_bytes), use_column_width=True)
                
                if st.button(f"Delete {item['name']}", key=f"delete_{idx}"):
                    api.delete_clothes(item['id'])
                    st.rerun()

    except Exception as e:
        st.error(f"Error loading clothes: {str(e)}")

def find_similar_page():
    st.header("Find Similar Clothes")
    
    uploaded_file = st.file_uploader("Upload an image to find similar clothes", type=['png', 'jpg', 'jpeg'])
    top_n = st.slider("Number of similar items to find", 1, 10, 5)
    
    if uploaded_file:
        temp_path = f"temp_search_{uploaded_file.name}"
        with open(temp_path, "wb") as f:
            f.write(uploaded_file.getbuffer())
            
        try:
            st.subheader("Your uploaded image:")
            st.image(uploaded_file, width=300)
            
            results = api.calculate_similarity(temp_path, top_n)
            
            st.subheader("Similar items found:")
            cols = st.columns(3)
            for idx, item in enumerate(results.get('data', [])):
                with cols[idx % 3]:
                    st.write(f"**{item['name']}**")
                    st.write(f"ID: {item['descriptions']}")
                    st.write(f"Descriptions: {item['descriptions']}")
                    # st.write(f"Similarity: {item['similarity']:.2f}")
                    
                    image_data = item.get('image', '')
                    if image_data:
                        import base64
                        from io import BytesIO
                        image_bytes = base64.b64decode(image_data)
                        st.image(BytesIO(image_bytes), use_column_width=True)
                        
        except Exception as e:
            st.error(f"Error finding similar items: {str(e)}")
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

if __name__ == "__main__":
    main()